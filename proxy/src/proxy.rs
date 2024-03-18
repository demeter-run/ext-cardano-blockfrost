use async_trait::async_trait;
use pingora::Result;
use pingora::{
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};
use pingora_limits::rate::Rate;
use regex::Regex;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::{Consumer, State, Tier};

static DMTR_API_KEY: &str = "dmtr-api-key";

pub struct BlockfrostProxy {
    state: Arc<State>,
    config: Arc<Config>,
    host_regex: Regex,
}
impl BlockfrostProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        let host_regex = Regex::new(r"(dmtr_[\w\d-]+)?\.?([\w]+)\.blockfrost-([\w\d]+).+").unwrap();

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
}

#[derive(Debug, Default)]
pub struct Context {
    instance: String,
    consumer: Consumer,
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

        let host = session
            .get_header("host")
            .map(|v| v.to_str().unwrap())
            .unwrap();

        let captures = self.host_regex.captures(host).unwrap();
        let network = captures.get(2).unwrap().as_str().to_string();

        let mut key = session
            .get_header(DMTR_API_KEY)
            .map(|v| v.to_str().unwrap())
            .unwrap_or_default();
        if let Some(m) = captures.get(1) {
            key = m.as_str();
        }

        let consumer = state.get_consumer(&network, key).await;
        if consumer.is_none() {
            return Err(pingora::Error::new(pingora::ErrorType::HTTPStatus(401)));
        }
        let consumer = consumer.unwrap();

        let instance = format!(
            "blockfrost-{network}.{}:{}",
            self.config.blockfrost_dns, self.config.blockfrost_port
        );

        if self.limiter(&consumer).await? {
            session.respond_error(429).await;
            return Ok(true);
        }

        *ctx = Context { instance, consumer };

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
}
