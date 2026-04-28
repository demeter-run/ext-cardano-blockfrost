use auth::AuthBackgroundService;
use cache_rules::{CacheRule, CacheRuleBackgroundService};
use config::Config;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use operator::kube::ResourceExt;
use operator::BlockfrostPort;
use pingora::{
    listeners::tls::TlsSettings,
    server::{
        configuration::{Opt, ServerConf},
        Server,
    },
    services::background::background_service,
};
use pingora_cache::eviction::simple_lru::Manager;
use pingora_limits::rate::Rate;
use prometheus::{histogram_opts, opts, register_histogram_vec, register_int_counter_vec};
use proxy::BlockfrostProxy;
use redb_storage::ReDbCache;
use regex::Regex;
use routing::background::RoutingBackgroundService;
use serde::{Deserialize, Deserializer};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tiers::TierBackgroundService;
use tokio::sync::RwLock;
use tracing::Level;

use crate::utils::handle_legacy_networks;

mod auth;
mod cache_rules;
mod config;
mod endpoints;
mod proxy;
mod redb_storage;
mod routing;
mod tiers;
mod utils;

static CACHE: Lazy<ReDbCache> = Lazy::new(|| ReDbCache::new(Config::new().cache_db_path));
static EVICTION: Lazy<Manager> = Lazy::new(|| Manager::new(Config::new().cache_max_size_bytes));

fn main() {
    dotenv().ok();

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let config: Arc<Config> = Arc::default();
    let state: Arc<State> = Arc::default();

    let opt = Opt::default();
    let server_conf = ServerConf {
        grace_period_seconds: Some(config.grace_period_seconds),
        graceful_shutdown_timeout_seconds: Some(config.graceful_shutdown_timeout_seconds),
        ..Default::default()
    };

    let mut server = Server::new_with_opt_and_conf(Some(opt), server_conf);
    server.bootstrap();

    let auth_background_service = background_service(
        "K8S Auth Service",
        AuthBackgroundService::new(state.clone()),
    );

    let cache_rules_background_service = background_service(
        "K8S Cache Rule Service",
        CacheRuleBackgroundService::new(state.clone(), config.clone()),
    );

    let tier_background_service = background_service(
        "K8S Tier Service",
        TierBackgroundService::new(state.clone(), config.clone()),
    );

    let routing_background_service = background_service(
        "Routing Service",
        RoutingBackgroundService::new(
            state.clone(),
            Arc::new(config.routing_config_path.clone()),
            config.routing_poll_interval,
        ),
    );
    let routing_background_service = server.add_service(routing_background_service);

    let auth_background_service = server.add_service(auth_background_service);
    let cache_rules_background_service = server.add_service(cache_rules_background_service);
    let tier_background_service = server.add_service(tier_background_service);

    let mut blockfrost_http_proxy = pingora::proxy::http_proxy_service(
        &server.configuration,
        BlockfrostProxy::new(state.clone(), config.clone()),
    );

    let mut tls_settings =
        TlsSettings::intermediate(&config.ssl_crt_path, &config.ssl_key_path).unwrap();

    // {
    //     use std::ops::DerefMut;
    //     let ctx = tls_settings.deref_mut().deref_mut();
    //     ctx.set_num_tickets(2).unwrap();
    //     ctx.set_max_early_data(16384).unwrap();
    // }

    tls_settings.enable_h2();

    blockfrost_http_proxy.add_tls_with_settings(&config.proxy_addr, None, tls_settings);
    let blockfrost_http_proxy = server.add_service(blockfrost_http_proxy);
    blockfrost_http_proxy.add_dependencies([
        &auth_background_service,
        &cache_rules_background_service,
        &tier_background_service,
        &routing_background_service,
    ]);

    let mut prometheus_service = pingora::services::listening::Service::prometheus_http_service();
    prometheus_service.add_tcp(&config.prometheus_addr);
    server.add_service(prometheus_service);

    server.run_forever();
}

#[derive(Default)]
pub struct State {
    consumers: RwLock<HashMap<String, Consumer>>,
    tiers: RwLock<HashMap<String, Tier>>,
    limiter: RwLock<HashMap<String, Vec<(TierRate, Rate)>>>,
    metrics: Metrics,
    cache_rules: RwLock<Vec<CacheRule>>,
    lifecycle: LifecycleState,
}
impl State {
    pub async fn get_consumer(&self, key: &str) -> Option<Consumer> {
        let consumers = self.consumers.read().await.clone();
        consumers.get(key).cloned()
    }

    pub fn get_cache() -> &'static ReDbCache {
        &CACHE
    }

    pub fn get_eviction() -> &'static Manager {
        &EVICTION
    }

    pub fn set_auth_ready(&self) {
        self.lifecycle.auth_ready.store(true, Ordering::Release);
    }

    pub fn set_tiers_ready(&self) {
        self.lifecycle.tiers_ready.store(true, Ordering::Release);
    }

    pub fn set_cache_rules_ready(&self) {
        self.lifecycle
            .cache_rules_ready
            .store(true, Ordering::Release);
    }

    pub fn set_routing_ready(&self) {
        self.lifecycle.routing_ready.store(true, Ordering::Release);
    }

    pub fn is_ready(&self) -> bool {
        self.lifecycle.auth_ready.load(Ordering::Acquire)
            && self.lifecycle.tiers_ready.load(Ordering::Acquire)
            && self.lifecycle.cache_rules_ready.load(Ordering::Acquire)
            && self.lifecycle.routing_ready.load(Ordering::Acquire)
    }
}

#[derive(Default)]
struct LifecycleState {
    auth_ready: AtomicBool,
    tiers_ready: AtomicBool,
    cache_rules_ready: AtomicBool,
    routing_ready: AtomicBool,
}

#[derive(Debug, Clone, Default)]
pub struct Consumer {
    namespace: String,
    port_name: String,
    tier: String,
    key: String,
    network: String,
}
impl Display for Consumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.namespace, self.port_name)
    }
}
impl From<&BlockfrostPort> for Consumer {
    fn from(value: &BlockfrostPort) -> Self {
        let network = handle_legacy_networks(&value.spec.network);
        let tier = value.spec.throughput_tier.to_string();
        let key = value.status.as_ref().unwrap().auth_token.clone();
        let namespace = value.metadata.namespace.as_ref().unwrap().clone();
        let port_name = value.name_any();

        Self {
            namespace,
            port_name,
            tier,
            key,
            network,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tier {
    name: String,
    rates: Vec<TierRate>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TierRate {
    limit: isize,
    #[serde(deserialize_with = "deserialize_duration")]
    interval: Duration,
}
pub fn deserialize_duration<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    let value: String = Deserialize::deserialize(deserializer)?;
    let regex = Regex::new(r"([\d]+)([\w])").unwrap();
    let captures = regex.captures(&value);
    if captures.is_none() {
        return Err(<D::Error as serde::de::Error>::custom(
            "Invalid tier interval format",
        ));
    }

    let captures = captures.unwrap();
    let number = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
    let symbol = captures.get(2).unwrap().as_str();

    match symbol {
        "s" => Ok(Duration::from_secs(number)),
        "m" => Ok(Duration::from_secs(number * 60)),
        "h" => Ok(Duration::from_secs(number * 60 * 60)),
        "d" => Ok(Duration::from_secs(number * 60 * 60 * 24)),
        _ => Err(<D::Error as serde::de::Error>::custom(
            "Invalid symbol tier interval",
        )),
    }
}

#[derive(Debug)]
pub struct Metrics {
    http_total_request: prometheus::IntCounterVec,
    http_request_duration_seconds: prometheus::HistogramVec,
}
impl Metrics {
    pub fn new() -> Self {
        let http_total_request = register_int_counter_vec!(
            opts!("blockfrost_proxy_http_total_request", "Total http request",),
            &[
                "consumer",
                "namespace",
                "instance",
                "status_code",
                "network",
                "tier",
            ]
        )
        .unwrap();

        let http_request_duration_seconds = register_histogram_vec!(
            histogram_opts!(
                "blockfrost_proxy_http_request_duration_seconds",
                "HTTP request duration in seconds",
                vec![
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0, 40.0,
                    60.0, 90.0, 120.0
                ]
            ),
            &["status_code", "network", "proxied", "resolved_by"]
        )
        .unwrap();

        Self {
            http_total_request,
            http_request_duration_seconds,
        }
    }

    pub fn inc_http_total_request(
        &self,
        consumer: &Consumer,
        namespace: &str,
        instance: &str,
        status: &u16,
    ) {
        self.http_total_request
            .with_label_values(&[
                &consumer.to_string(),
                namespace,
                instance,
                &status.to_string(),
                &consumer.network,
                &consumer.tier,
            ])
            .inc()
    }
    /// Observe HTTP request duration in seconds.
    pub fn observe_http_request_duration(
        &self,
        consumer: &Consumer,
        status: &u16,
        proxied: bool,
        duration: std::time::Duration,
        resolved_by: String,
    ) {
        self.http_request_duration_seconds
            .with_label_values(&[
                &status.to_string(),
                &consumer.network,
                &proxied.to_string(),
                &resolved_by.to_string(),
            ])
            .observe(duration.as_secs_f64());
    }
}
impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
