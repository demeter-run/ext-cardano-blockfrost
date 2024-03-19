use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_addr: String,
    pub proxy_namespace: String,
    pub proxy_tiers_path: PathBuf,
    pub prometheus_addr: String,
    pub ssl_crt_path: String,
    pub ssl_key_path: String,
    pub blockfrost_port: u16,
    pub blockfrost_dns: String,

    // Cache settings
    pub cache_rules_path: PathBuf,
    pub cache_db_path: String,
}
impl Config {
    pub fn new() -> Self {
        Self {
            proxy_addr: env::var("PROXY_ADDR").expect("PROXY_ADDR must be set"),
            proxy_namespace: env::var("PROXY_NAMESPACE").expect("PROXY_NAMESPACE must be set"),
            proxy_tiers_path: env::var("PROXY_TIERS_PATH")
                .map(|v| v.into())
                .expect("PROXY_TIERS_PATH must be set"),
            prometheus_addr: env::var("PROMETHEUS_ADDR").expect("PROMETHEUS_ADDR must be set"),
            ssl_crt_path: env::var("SSL_CRT_PATH").expect("SSL_CRT_PATH must be set"),
            ssl_key_path: env::var("SSL_KEY_PATH").expect("SSL_KEY_PATH must be set"),
            blockfrost_port: env::var("BLOCKFROST_PORT")
                .expect("BLOCKFROST_PORT must be set")
                .parse()
                .expect("BLOCKFROST_PORT must a number"),
            blockfrost_dns: env::var("BLOCKFROST_DNS").expect("BLOCKFROST_DNS must be set"),
            cache_rules_path: env::var("CACHE_RULES_PATH")
                .map(|v| v.into())
                .expect("PROXY_TIERS_PATH must be set"),
            cache_db_path: env::var("CACHE_DB_PATH").expect("CACHE_DB_PATH must be set"),
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
