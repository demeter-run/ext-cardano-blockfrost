use crate::routing::{Backend, ROUTER};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use pingora::http::{RequestHeader, ResponseHeader, StatusCode};
use pingora::{
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};
use pingora::{Error, Result};
use pingora_cache::{CacheKey, CacheMeta, ForcedFreshness, HitHandler, RespCacheable};
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

fn resolve_backend_for_config(config: &Config, network: &str, path: &str) -> Backend {
    let router = ROUTER.load();
    let backend = router.resolve(path);

    match backend {
        Backend::Dolos => {
            if should_use_dolos(config, network, path) {
                Backend::Dolos
            } else {
                Backend::Blockfrost
            }
        }
        Backend::Blockfrost | Backend::SubmitApi => backend,
    }
}

fn format_instance_for_config(backend: Backend, network: &str) -> String {
    let router = ROUTER.load();
    let template = router.backend_template(backend);
    template.replace("{network}", network)
}

fn should_use_dolos(config: &Config, network: &str, path: &str) -> bool {
    !network.starts_with("vector") && config.dolos_enabled && !is_byron_block_path(path)
}

fn is_byron_block_path(path: &str) -> bool {
    let mut segments = path.trim_start_matches('/').split('/');
    if segments.next() != Some("blocks") {
        return false;
    }

    let hash_or_number = match segments.next() {
        Some(value) => value,
        None => return false,
    };

    if !hash_or_number.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }

    if hash_or_number.len() != 64 {
        if let Ok(number) = hash_or_number.parse::<u32>() {
            return number <= LAST_BYRON_BLOCK;
        }
    }

    false
}

pub struct BlockfrostProxy {
    state: Arc<State>,
    config: Arc<Config>,
    host_regex: Regex,
}

impl BlockfrostProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        let host_regex = Regex::new(r"([dmtr_]?[\w\d-]+)?\.?.+").unwrap();

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
            .and_then(|v| v.to_str().ok())
            .or_else(|| session.req_header().uri.authority().map(|a| a.as_str()))
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
        self.respond_status(session, ctx, 200, "OK").await;
    }

    async fn respond_readiness(&self, session: &mut Session, ctx: &mut Context) {
        let is_ready = self.state.is_ready() && !session.is_process_shutting_down();
        let status = if is_ready { 200 } else { 503 };
        let body = if is_ready { "READY" } else { "NOT READY" };

        self.respond_status(session, ctx, status, body).await;
    }

    async fn respond_status(
        &self,
        session: &mut Session,
        ctx: &mut Context,
        status: u16,
        body: &str,
    ) {
        ctx.is_probe_request = true;
        session.set_keepalive(None);
        let header = Box::new(ResponseHeader::build(status, None).unwrap());
        session.write_response_header(header, false).await.unwrap();
        session
            .write_response_body(Some(body.to_string().into()), true)
            .await
            .unwrap();
    }
}

#[derive(Debug, Default)]
pub struct Context {
    instance: String,
    consumer: Consumer,
    cache_rule: Option<CacheRule>,
    endpoint: String,
    is_probe_request: bool,
    start_time: Option<Instant>,
    resolved_by: String,
    tries: usize,
}

#[async_trait]
impl ProxyHttp for BlockfrostProxy {
    type CTX = Context;
    fn new_ctx(&self) -> Self::CTX {
        Context::default()
    }

    fn fail_to_connect(
        &self,
        _session: &mut Session,
        _peer: &HttpPeer,
        ctx: &mut Self::CTX,
        mut e: Box<Error>,
    ) -> Box<Error> {
        if ctx.tries >= self.config.max_retries {
            return e;
        }
        ctx.tries += 1;
        e.set_retry(true);
        e
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

        if path == self.config.readiness_endpoint {
            self.respond_readiness(session, ctx).await;
            return Ok(true);
        }

        if session.is_process_shutting_down() {
            session.set_keepalive(None);
            let _ = session.respond_error(503).await;
            return Ok(true);
        }

        if self.is_forbidden_endpoint(path) {
            dbg!(path);
            let _ = session.respond_error(501).await;
            return Ok(true);
        }

        let key = self.extract_key(session);
        let consumer = state.get_consumer(&key).await;
        if consumer.is_none() {
            let _ = session.respond_error(401).await;
            return Ok(true);
        }

        ctx.consumer = consumer.unwrap();

        let backend = resolve_backend_for_config(self.config.as_ref(), &ctx.consumer.network, path);
        ctx.instance = format_instance_for_config(backend, &ctx.consumer.network);
        ctx.resolved_by = backend.as_str().to_string();

        if self.limiter(&ctx.consumer).await? {
            let _ = session.respond_error(429).await;
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
        let mut http_peer = HttpPeer::new(&ctx.instance, false, String::default());
        http_peer.options.connection_timeout = Some(self.config.connection_timeout);
        Ok(Box::new(http_peer))
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // Modify the path based on the resolved_by backend
        if ctx.resolved_by == "submitapi" {
            // We know the original path is /tx/submit
            // Set the right path for Submit API
            upstream_request.set_uri("/api/submit/tx".parse().unwrap());
        }

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        if !ctx.is_probe_request {
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
            session.cache.enable(
                State::get_cache(),
                Some(State::get_eviction()),
                None,
                None,
                None,
            );
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
        _session: &mut Session,
        _meta: &CacheMeta,
        _hit_handler: &mut HitHandler,
        _is_fresh: bool,
        ctx: &mut Self::CTX,
    ) -> Result<Option<ForcedFreshness>>
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
        Ok(None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::{BackendTemplateConfig, RouteConfig, RoutingConfig};
    use once_cell::sync::Lazy;
    use std::path::PathBuf;
    use std::sync::Mutex;

    static ROUTER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn base_config() -> Config {
        Config {
            proxy_addr: "0.0.0.0:0".to_string(),
            proxy_namespace: "proxy".to_string(),
            proxy_tiers_path: PathBuf::from("/tmp"),
            proxy_tiers_poll_interval: Duration::from_secs(1),
            prometheus_addr: "0.0.0.0:0".to_string(),
            ssl_crt_path: "crt".to_string(),
            ssl_key_path: "key".to_string(),
            dolos_enabled: true,
            routing_config_path: PathBuf::from("/tmp/routing.toml"),
            routing_poll_interval: Duration::from_secs(1),
            cache_rules_path: PathBuf::from("/tmp"),
            cache_db_path: "cache".to_string(),
            cache_failed_requests_seconds: 5,
            cache_max_size_bytes: 1024,
            forbidden_endpoints: vec![],
            health_endpoint: "/health".to_string(),
            readiness_endpoint: "/ready".to_string(),
            grace_period_seconds: 30,
            graceful_shutdown_timeout_seconds: 5,
            max_retries: 3,
            connection_timeout: Duration::from_secs(1),
        }
    }

    #[test]
    fn byron_blocks_stay_on_blockfrost() {
        let _guard = ROUTER_LOCK.lock().unwrap();
        let cfg = RoutingConfig {
            default_backend: "blockfrost".to_string(),
            backend_templates: BackendTemplateConfig::default(),
            routes: vec![
                RouteConfig {
                    path: "/blocks/{hash}".to_string(),
                    backend: "dolos".to_string(),
                },
                RouteConfig {
                    path: "/tx/submit".to_string(),
                    backend: "submitapi".to_string(),
                },
            ],
        };
        let router = cfg.build_router().unwrap();
        ROUTER.store(Arc::new(router));

        let config = base_config();
        assert_eq!(
            resolve_backend_for_config(&config, "cardano-mainnet", "/blocks/4490000"),
            Backend::Blockfrost
        );
        assert_eq!(
            resolve_backend_for_config(&config, "cardano-mainnet", "/blocks/4490511"),
            Backend::Dolos
        );
    }

    #[test]
    fn template_interpolation_uses_config() {
        let _guard = ROUTER_LOCK.lock().unwrap();
        let cfg = RoutingConfig {
            default_backend: "blockfrost".to_string(),
            backend_templates: BackendTemplateConfig {
                blockfrost: "bf-{network}:3000".to_string(),
                dolos: "dolos-{network}:50051".to_string(),
                submitapi: "submit-{network}:8090".to_string(),
            },
            routes: vec![RouteConfig {
                path: "/tx/submit".to_string(),
                backend: "submitapi".to_string(),
            }],
        };
        let router = cfg.build_router().unwrap();
        ROUTER.store(Arc::new(router));

        assert_eq!(
            format_instance_for_config(Backend::SubmitApi, "cardano-mainnet"),
            "submit-cardano-mainnet:8090"
        );
    }
}
