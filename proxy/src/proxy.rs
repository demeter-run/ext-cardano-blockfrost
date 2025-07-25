use async_trait::async_trait;
use once_cell::sync::Lazy;
use pingora::http::{RequestHeader, ResponseHeader, StatusCode};
use pingora::Result;
use pingora::{
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};
use pingora_cache::{CacheKey, CacheMeta, RespCacheable};
use pingora_limits::rate::Rate;
use prometheus::{register_int_counter_vec, IntCounterVec};
use regex::Regex;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tracing::info;

use crate::cache_rules::CacheRule;
use crate::config::Config;
use crate::{Consumer, State, Tier};

static DMTR_API_KEY: &str = "dmtr-api-key";
static CACHE_HIT_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "blockfrost_proxy_http_cache_hits",
        "Number of times cache was used.",
        &["endpoint", "network", "project", "resolved_by"]
    )
    .unwrap()
});
static CACHE_MISS_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "blockfrost_proxy_http_cache_miss",
        "Number of times cache was requested, but no entry was found.",
        &["endpoint", "network", "project", "resolved_by"]
    )
    .unwrap()
});
static LAST_BYRON_BLOCK: u32 = 4490510;

pub struct BlockfrostProxy {
    state: Arc<State>,
    config: Arc<Config>,
    host_regex: Regex,
    blocks_endpoint_regex: Regex,
}

impl BlockfrostProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        let host_regex = Regex::new(r"([dmtr_]?[\w\d-]+)?\.?.+").unwrap();
        let blocks_endpoint_regex = Regex::new(r"^\/blocks\/([A-z0-9]+)\/?\w*$").unwrap();

        Self {
            state,
            config,
            host_regex,
            blocks_endpoint_regex,
        }
    }

    async fn has_limiter(&self, consumer: &Consumer) -> bool {
        let rate_limiter_map = self.state.limiter.read().await;
        rate_limiter_map.get(&consumer.key).is_some()
    }

    async fn add_limiter(&self, consumer: &Consumer, tier: &Tier) {
        let rates = tier
            .rates
            .iter()
            .map(|r| (r.clone(), Rate::new(r.interval)))
            .collect();

        self.state
            .limiter
            .write()
            .await
            .insert(consumer.key.clone(), rates);
    }

    async fn limiter(&self, consumer: &Consumer) -> Result<bool> {
        let tiers = self.state.tiers.read().await.clone();
        let tier = tiers.get(&consumer.tier);
        if tier.is_none() {
            return Ok(true);
        }
        let tier = tier.unwrap();

        if !self.has_limiter(consumer).await {
            self.add_limiter(consumer, tier).await;
        }

        let rate_limiter_map = self.state.limiter.read().await;
        let rates = rate_limiter_map.get(&consumer.key).unwrap();

        if rates
            .iter()
            .any(|(t, r)| r.observe(&consumer.key, 1) > t.limit)
        {
            return Ok(true);
        }

        Ok(false)
    }

    fn extract_key(&self, session: &Session) -> String {
        let host = session
            .get_header("host")
            .map(|v| v.to_str().unwrap())
            .unwrap();

        let captures = self.host_regex.captures(host).unwrap();

        let key = session
            .get_header(DMTR_API_KEY)
            .and_then(|v| v.to_str().ok())
            .or_else(|| captures.get(1).map(|v| v.as_str()))
            .unwrap_or_default();

        key.to_string()
    }

    fn is_forbidden_endpoint(&self, path: &str) -> bool {
        for forbidden_endpoint in self.config.forbidden_endpoints.clone().into_iter() {
            if forbidden_endpoint.matches(path) {
                return true;
            }
        }
        false
    }

    async fn get_rule(&self, path: &str) -> Option<CacheRule> {
        let rules = self.state.cache_rules.read().await.clone();
        for rule in rules.into_iter() {
            if rule.matches(path) {
                return Some(rule);
            }
        }
        None
    }

    async fn respond_health(&self, session: &mut Session, ctx: &mut Context) {
        ctx.is_health_request = true;
        session.set_keepalive(None);
        session.write_response_body("OK".into()).await.unwrap();
        let header = Box::new(ResponseHeader::build(200, None).unwrap());
        session.write_response_header(header).await.unwrap();
    }

    async fn respond_with_static_params(&self, session: &mut Session, _ctx: &mut Context) {
        let body = include_str!("params.json");

        let mut header = ResponseHeader::build(200, None).unwrap();

        header
            .insert_header("access-control-allow-origin", "*")
            .unwrap();

        header
            .insert_header("content-type", "application/json; charset=utf-8")
            .unwrap();

        header
            .insert_header("content-length", body.len().to_string())
            .unwrap();

        session
            .write_response_header(Box::new(header))
            .await
            .unwrap();

        session.write_response_body(body.into()).await.unwrap();

        session.finish_body().await.unwrap();
    }

    fn is_dolos_path(&self, path: &str) -> bool {
        // ADHOC: Old blocks should go to DBSync
        if let Some(captures) = self.blocks_endpoint_regex.captures(path) {
            if let Some(hash_or_number) = captures.get(1) {
                let hash_or_number = hash_or_number.as_str();
                if hash_or_number.len() != 64 {
                    if let Ok(number) = hash_or_number.parse::<u32>() {
                        if number <= LAST_BYRON_BLOCK {
                            return false;
                        }
                    }
                }
            }
        }

        for dolos_endpoint in self.config.dolos_endpoints.clone().into_iter() {
            if dolos_endpoint.matches(path) {
                return true;
            }
        }
        false
    }

    fn should_use_dolos(&self, path: &str) -> bool {
        self.config.dolos_enabled && self.is_dolos_path(path)
    }
}

#[derive(Debug, Default)]
pub struct Context {
    instance: String,
    consumer: Consumer,
    cache_rule: Option<CacheRule>,
    endpoint: String,
    is_health_request: bool,
    start_time: Option<Instant>,
    resolved_by: String,
}

#[async_trait]
impl ProxyHttp for BlockfrostProxy {
    type CTX = Context;
    fn new_ctx(&self) -> Self::CTX {
        Context::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        ctx.start_time = Some(Instant::now());
        let state = self.state.clone();

        let path = session.req_header().uri.path();

        if path == self.config.health_endpoint {
            self.respond_health(session, ctx).await;
            return Ok(true);
        }

        if self.is_forbidden_endpoint(path) {
            dbg!(path);
            session.respond_error(501).await;
            return Ok(true);
        }

        let key = self.extract_key(session);
        let consumer = state.get_consumer(&key).await;
        if consumer.is_none() {
            session.respond_error(401).await;
            return Ok(true);
        }

        ctx.consumer = consumer.unwrap();

        if self.should_use_dolos(path) {
            ctx.instance = format!(
                //eg: internal-cardano-mainnet-minibf.ext-utxorpc-m1.svc.cluster.local:3001
                "internal-cardano-{}-minibf.{}:{}",
                ctx.consumer.network, self.config.dolos_dns, self.config.dolos_port
            );
            ctx.resolved_by = "dolos".to_string();
        } else {
            ctx.instance = format!(
                "blockfrost-{}.{}:{}",
                ctx.consumer.network, self.config.blockfrost_dns, self.config.blockfrost_port
            );
            ctx.resolved_by = "blockfrost".to_string();
        }

        if self.limiter(&ctx.consumer).await? {
            session.respond_error(429).await;
            return Ok(true);
        }

        // TODO: this is a temporary fix while we migrate dbsync
        if path == "/epochs/latest/parameters" {
            self.respond_with_static_params(session, ctx).await;
            return Ok(true);
        }

        let cache_rule = self.get_rule(path).await;
        ctx.cache_rule = cache_rule;
        ctx.endpoint = path.to_string();

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let http_peer = HttpPeer::new(&ctx.instance, false, String::default());
        Ok(Box::new(http_peer))
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        if !ctx.is_health_request {
            let response_code = session
                .response_written()
                .map_or(0, |resp| resp.status.as_u16());

            self.state.metrics.inc_http_total_request(
                &ctx.consumer,
                &self.config.proxy_namespace,
                &ctx.instance,
                &response_code,
            );
            if let Some(start) = ctx.start_time {
                let dur = start.elapsed();

                self.state.metrics.observe_http_request_duration(
                    &ctx.consumer,
                    &response_code,
                    ctx.cache_rule.is_some(),
                    dur,
                    ctx.resolved_by.clone(),
                );
                info!(
                    response_time = dur.as_millis(),
                    "{} response code: {response_code}",
                    self.request_summary(session, ctx)
                );
            } else {
                info!(
                    "{} response code: {response_code}",
                    self.request_summary(session, ctx)
                );
            }
        }
    }

    // Cache related stuff

    /// Build cache key from the request.
    fn cache_key_callback(&self, session: &Session, ctx: &mut Self::CTX) -> Result<CacheKey> {
        let req_header = session.req_header();
        Ok(CacheKey::new(
            ctx.consumer.network.clone(),
            format!(
                "{}{}",
                req_header.uri.path(),
                req_header.uri.query().unwrap_or("")
            ),
            "".to_string(),
        ))
    }

    fn request_cache_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<()> {
        if ctx.cache_rule.is_some() {
            session
                .cache
                .enable(State::get_cache(), Some(State::get_eviction()), None, None);
        }
        Ok(())
    }

    fn response_cache_filter(
        &self,
        _session: &Session,
        resp: &ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<RespCacheable> {
        let rule = ctx
            .cache_rule
            .clone()
            .expect("Cache rule unexpectedly None.");

        let cache_seconds = match resp.status {
            StatusCode::OK => rule.duration_s,
            _ => self.config.cache_failed_requests_seconds,
        };

        Ok(RespCacheable::Cacheable(CacheMeta::new(
            SystemTime::now()
                .checked_add(Duration::new(cache_seconds, 0))
                .unwrap(),
            SystemTime::now(),
            0,
            0,
            resp.clone(),
        )))
    }

    async fn cache_hit_filter(
        &self,
        _meta: &CacheMeta,
        ctx: &mut Self::CTX,
        _req: &RequestHeader,
    ) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let _ = &CACHE_HIT_COUNTER
            .with_label_values(&[
                &ctx.cache_rule.clone().unwrap().endpoint.to_string(),
                &ctx.consumer.network,
                &ctx.consumer.namespace,
                &ctx.resolved_by,
            ])
            .inc();
        Ok(false)
    }

    fn cache_miss(&self, session: &mut Session, ctx: &mut Self::CTX) {
        let _ = &CACHE_MISS_COUNTER
            .with_label_values(&[
                &ctx.cache_rule.clone().unwrap().endpoint.to_string(),
                &ctx.consumer.network,
                &ctx.consumer.namespace,
                &ctx.resolved_by,
            ])
            .inc();
        session.cache.cache_miss();
    }
}
