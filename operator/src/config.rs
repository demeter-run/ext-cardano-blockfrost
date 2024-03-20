use lazy_static::lazy_static;
use std::{collections::HashMap, env, time::Duration};

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dns_zone: String,
    pub extension_subdomain: String,
    pub api_key_salt: String,
    pub metrics_delay: Duration,
    pub prometheus_url: String,
    pub dcu_per_request: HashMap<String, f64>,
    pub default_blockfrost_version: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            dns_zone: env::var("DNS_ZONE").unwrap_or("demeter.run".into()),
            extension_subdomain: env::var("EXTENSION_SUBDOMAIN").unwrap_or("blockfrost-m1".into()),
            api_key_salt: env::var("API_KEY_SALT").unwrap_or("blockfrost-salt".into()),
            metrics_delay: Duration::from_secs(
                std::env::var("METRICS_DELAY")
                    .expect("METRICS_DELAY must be set")
                    .parse::<u64>()
                    .expect("METRICS_DELAY must be a number"),
            ),
            prometheus_url: env::var("PROMETHEUS_URL").expect("PROMETHEUS_URL must be set"),
            dcu_per_request: env::var("DCU_PER_REQUEST")
                .expect("DCU_PER_REQUEST must be set")
                .split(',')
                .map(|pair| {
                    let parts: Vec<&str> = pair.split('=').collect();
                    let dcu = parts[1]
                        .parse::<f64>()
                        .expect("DCU_PER_REQUEST must be NETWORK=NUMBER");

                    (parts[0].into(), dcu)
                })
                .collect(),
            default_blockfrost_version: env::var("default_blockfrost_version")
                .unwrap_or("v1".into()),
        }
    }
}
