use lazy_static::lazy_static;
use std::{env, time::Duration};

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dns_zone: String,

    pub ingress_class: String,
    pub api_key_salt: String,

    pub metrics_delay: Duration,
    pub prometheus_url: String,
    pub dcu_per_request_mainnet: f64,
    pub dcu_per_request_preprod: f64,
    pub dcu_per_request_preview: f64,
    pub dcu_per_request_sanchonet: f64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            dns_zone: env::var("DNS_ZONE").unwrap_or("demeter.run".into()),
            ingress_class: env::var("INGRESS_CLASS").unwrap_or("blockfrost-v1".into()),
            api_key_salt: env::var("API_KEY_SALT").expect("API_KEY_SALT must be set"),

            metrics_delay: Duration::from_secs(
                std::env::var("METRICS_DELAY")
                    .expect("METRICS_DELAY must be set")
                    .parse::<u64>()
                    .expect("METRICS_DELAY must be a number"),
            ),
            prometheus_url: env::var("PROMETHEUS_URL").expect("PROMETHEUS_URL must be set"),
            dcu_per_request_mainnet: std::env::var("DCU_PER_REQUEST_MAINNET")
                .expect("DCU_PER_REQUEST_MAINNET must be set")
                .parse::<f64>()
                .expect("DCU_PER_REQUEST_MAINNET must be a number"),
            dcu_per_request_preprod: std::env::var("DCU_PER_REQUEST_PREPROD")
                .expect("DCU_PER_REQUEST_PREPROD must be set")
                .parse::<f64>()
                .expect("DCU_PER_REQUEST_PREPROD must be a number"),
            dcu_per_request_preview: std::env::var("DCU_PER_REQUEST_PREVIEW")
                .expect("DCU_PER_REQUEST_PREVIEW must be set")
                .parse::<f64>()
                .expect("DCU_PER_REQUEST_PREVIEW must be a number"),
            dcu_per_request_sanchonet: std::env::var("DCU_PER_REQUEST_SANCHONET")
                .expect("DCU_PER_REQUEST_SANCHONET must be set")
                .parse::<f64>()
                .expect("DCU_PER_REQUEST_SANCHONET must be a number"),
        }
    }
}
