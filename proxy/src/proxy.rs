use async_trait::async_trait;
use once_cell::sync::Lazy;
use pingora::http::{RequestHeader, ResponseHeader};
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
use std::time::{Duration, SystemTime};
use tracing::info;

use crate::cache_rules::CacheRule;
use crate::config::Config;
use crate::{Consumer, State, Tier};

static DMTR_API_KEY: &str = "dmtr-api-key";
static CACHE_HIT_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "blockfrost_proxy_http_cache_hits",
        "Number of times cache was used.",
        &["endpoint", "network", "project"]
    )
    .unwrap()
});
static CACHE_MISS_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "blockfrost_proxy_http_cache_miss",
        "Number of times cache was requested, but no entry was found.",
        &["endpoint", "network", "project"]
    )
    .unwrap()
});

pub struct BlockfrostProxy {
    state: Arc<State>,
    config: Arc<Config>,
    host_regex: Regex,
}
impl BlockfrostProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        let host_regex = Regex::new(r"(dmtr_[\w\d-]+)?\.?.+").unwrap();

        Self {
            state,
            config,
            host_regex,
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
        let mut key = session
            .get_header(DMTR_API_KEY)
            .map(|v| v.to_str().unwrap())
            .unwrap_or_default();
        if let Some(m) = captures.get(1) {
            key = m.as_str();
        }
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
}

#[derive(Debug, Default)]
pub struct Context {
    instance: String,
    consumer: Consumer,
    cache_rule: Option<CacheRule>,
    endpoint: String,
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
        let state = self.state.clone();

        let path = session.req_header().uri.path();
        if self.is_forbidden_endpoint(path) {
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
        ctx.instance = format!(
            "blockfrost-{}.{}:{}",
            ctx.consumer.network, self.config.blockfrost_dns, self.config.blockfrost_port
        );
        ctx.instance = "localhost:3000".to_string();

        if self.limiter(&ctx.consumer).await? {
            session.respond_error(429).await;
            return Ok(true);
        }

        let cache_rule = self.get_rule(path).await;
        ctx.cache_rule = cache_rule;
        ctx.endpoint = path.to_string().clone();

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
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());

        info!(
            "{} response code: {response_code}",
            self.request_summary(session, ctx)
        );

        self.state.metrics.inc_http_total_request(
            &ctx.consumer,
            &self.config.proxy_namespace,
            &ctx.instance,
            &response_code,
        );
    }

    // Cache related stuff

    /// Build cache key from the request.
    fn cache_key_callback(&self, session: &Session, ctx: &mut Self::CTX) -> Result<CacheKey> {
        let req_header = session.req_header();
        Ok(CacheKey::new(
            ctx.consumer.network.clone(),
            req_header.uri.path(),
            "".to_string(),
        ))
    }

    fn request_cache_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<()> {
        if ctx.cache_rule.is_some() {
            session.cache.enable(State::get_cache(), None, None, None);
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

        Ok(RespCacheable::Cacheable(CacheMeta::new(
            SystemTime::now()
                .checked_add(Duration::new(rule.duration_s, 0))
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
            ])
            .inc();
        session.cache.cache_miss();
    }
}
