use std::{env, path::PathBuf, time::Duration};

use crate::endpoints::Endpoint;

#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_addr: String,
    pub proxy_namespace: String,
    pub proxy_tiers_path: PathBuf,
    pub proxy_tiers_poll_interval: Duration,
    pub prometheus_addr: String,
    pub ssl_crt_path: String,
    pub ssl_key_path: String,
    pub blockfrost_port: u16,
    pub blockfrost_dns: String,

    // Dolos settings
    pub dolos_enabled: bool,
    pub dolos_port: u16,
    pub dolos_dns: String,
    pub dolos_endpoints: Vec<Endpoint>,

    // Cache settings
    pub cache_rules_path: PathBuf,
    pub cache_db_path: String,
    pub cache_failed_requests_seconds: u64,
    pub cache_max_size_bytes: usize,

    // Forbidden endpoints
    pub forbidden_endpoints: Vec<Endpoint>,

    // Health endpoint
    pub health_endpoint: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            proxy_addr: env::var("PROXY_ADDR").expect("PROXY_ADDR must be set"),
            proxy_namespace: env::var("PROXY_NAMESPACE").expect("PROXY_NAMESPACE must be set"),
            proxy_tiers_path: env::var("PROXY_TIERS_PATH")
                .map(|v| v.into())
                .expect("PROXY_TIERS_PATH must be set"),
            proxy_tiers_poll_interval: env::var("PROXY_TIERS_POLL_INTERVAL")
                .map(|v| {
                    Duration::from_secs(
                        v.parse::<u64>()
                            .expect("PROXY_TIERS_POLL_INTERVAL must be a number in seconds. eg: 2"),
                    )
                })
                .unwrap_or(Duration::from_secs(2)),
            prometheus_addr: env::var("PROMETHEUS_ADDR").expect("PROMETHEUS_ADDR must be set"),
            ssl_crt_path: env::var("SSL_CRT_PATH").expect("SSL_CRT_PATH must be set"),
            ssl_key_path: env::var("SSL_KEY_PATH").expect("SSL_KEY_PATH must be set"),
            blockfrost_port: env::var("BLOCKFROST_PORT")
                .expect("BLOCKFROST_PORT must be set")
                .parse()
                .expect("BLOCKFROST_PORT must a number"),
            blockfrost_dns: env::var("BLOCKFROST_DNS").expect("BLOCKFROST_DNS must be set"),
            dolos_enabled: env::var("DOLOS_ENABLED").unwrap_or("false".to_string()) == "true",
            dolos_port: env::var("DOLOS_PORT")
                .expect("DOLOS_PORT must be set")
                .parse()
                .expect("DOLOS_PORT must a number"),
            dolos_dns: env::var("DOLOS_DNS").expect("DOLOS_DNS must be set"),
            cache_rules_path: env::var("CACHE_RULES_PATH")
                .map(|v| v.into())
                .expect("CACHE_RULES_PATH must be set"),
            cache_db_path: env::var("CACHE_DB_PATH").expect("CACHE_DB_PATH must be set"),
            cache_failed_requests_seconds: env::var("CACHE_FAILED_REQUESTS_SECONDS")
                .unwrap_or("20".to_string())
                .parse()
                .expect("CACHE_FAILED_REQUESTS_SECONDS must a number"),
            cache_max_size_bytes: env::var("CACHE_MAX_SIZE_BYTES")
                .unwrap_or("3000000".to_string())
                .parse()
                .expect("CACHE_MAX_SIZE_BYTES must a number"),
            forbidden_endpoints: env::var("FORBIDDEN_ENDPOINTS")
                .unwrap_or("".into())
                .split(',')
                .map(|endpoint| Endpoint::new(endpoint).expect("Invalid forbidden endpoint regex"))
                .collect(),
            dolos_endpoints: env::var("DOLOS_ENDPOINTS")
                .expect("Missing DOLOS_ENDPOINTS variable")
                .split(',')
                .map(|endpoint| Endpoint::new(endpoint).expect("Invalid dolos endpoint regex"))
                .collect(),
            health_endpoint: "/dmtr_health".to_string(),
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_from_env() {
        let file1 = NamedTempFile::new().unwrap();
        let path = file1.path().to_str().unwrap();

        env::set_var("PROXY_ADDR", "0.0.0.0:8000");
        env::set_var("PROXY_NAMESPACE", "namespace");
        env::set_var("PROXY_TIERS_PATH", path);
        env::set_var("PROMETHEUS_ADDR", "0.0.0.0:8001");
        env::set_var("SSL_CRT_PATH", "ssl_crt_path");
        env::set_var("SSL_KEY_PATH", "ssl_key_path");
        env::set_var("BLOCKFROST_PORT", "3000");
        env::set_var("BLOCKFROST_DNS", "ext-blockfrost-m1");
        env::set_var("CACHE_RULES_PATH", path);
        env::set_var("CACHE_DB_PATH", path);
        env::set_var("FORBIDDEN_ENDPOINTS", r"/network,/pools/\w+$");
        env::set_var("DOLOS_PORT", "50051");
        env::set_var("DOLOS_DNS", "ext-utxorpc-m1");

        let config = Config::new();
        assert!(config.forbidden_endpoints[0].matches("/network"));
        assert!(config.forbidden_endpoints[1].matches("/pools/pool_id"));
        assert!(!config.forbidden_endpoints[1].matches("/pools/pool_id/blocks"));
    }
}
