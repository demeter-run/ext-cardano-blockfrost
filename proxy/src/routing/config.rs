use serde::Deserialize;

use super::router::Router;
use super::trie::RouteTrie;
use super::{Backend, RoutingError};

#[derive(Debug, Deserialize, Clone)]
pub struct RoutingConfig {
    #[serde(default = "default_backend")]
    pub default_backend: String,

    #[serde(default)]
    pub backends: BackendsConfig,

    #[serde(default)]
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub backend: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub template: String,

    #[serde(default)]
    pub supported_networks: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendsConfig {
    pub blockfrost: BackendConfig,
    pub dolos: BackendConfig,
    pub submitapi: BackendConfig,
}

fn default_backend() -> String {
    "blockfrost".to_string()
}

impl BackendConfig {
    pub(crate) fn supports_network(&self, network: &str) -> bool {
        self.supported_networks.is_empty()
            || self.supported_networks.iter().any(|value| value == network)
    }
}

impl Default for BackendsConfig {
    fn default() -> Self {
        Self {
            blockfrost: BackendConfig {
                template: "blockfrost-{network}:3000".to_string(),
                supported_networks: vec![],
            },
            dolos: BackendConfig {
                template: "internal-{network}-minibf:50051".to_string(),
                supported_networks: default_supported_networks(),
            },
            submitapi: BackendConfig {
                template: "submitapi-{network}:8090".to_string(),
                supported_networks: default_supported_networks(),
            },
        }
    }
}

fn default_supported_networks() -> Vec<String> {
    vec![
        "cardano-mainnet".to_string(),
        "cardano-preprod".to_string(),
        "cardano-preview".to_string(),
    ]
}

impl RoutingConfig {
    pub fn build_router(&self) -> Result<Router, RoutingError> {
        let default_backend = Backend::from_str(&self.default_backend)?;
        let mut trie = RouteTrie::new();

        for route in &self.routes {
            let backend = Backend::from_str(&route.backend)?;
            trie.insert(&route.path, backend)?;
        }

        Ok(Router::new(default_backend, trie, self.backends.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backend_applied() {
        let cfg = RoutingConfig {
            default_backend: "blockfrost".into(),
            backends: BackendsConfig::default(),
            routes: vec![],
        };
        let router = cfg.build_router().unwrap();
        assert_eq!(router.resolve("/unknown"), Backend::Blockfrost);
    }

    #[test]
    fn dolos_route_resolves() {
        let cfg = RoutingConfig {
            default_backend: "blockfrost".into(),
            backends: BackendsConfig::default(),
            routes: vec![RouteConfig {
                path: "/blocks/{hash}".into(),
                backend: "dolos".into(),
            }],
        };
        let router = cfg.build_router().unwrap();
        assert_eq!(router.resolve("/blocks/abc"), Backend::Dolos);
    }

    #[test]
    fn backend_config_passthrough() {
        let cfg = RoutingConfig {
            default_backend: "blockfrost".into(),
            backends: BackendsConfig {
                blockfrost: BackendConfig {
                    template: "bf-{network}".into(),
                    supported_networks: vec![],
                },
                dolos: BackendConfig {
                    template: "dolos-{network}".into(),
                    supported_networks: vec!["cardano-mainnet".into()],
                },
                submitapi: BackendConfig {
                    template: "submit-{network}".into(),
                    supported_networks: vec!["cardano-mainnet".into()],
                },
            },
            routes: vec![],
        };
        let router = cfg.build_router().unwrap();
        assert_eq!(router.backend_template(Backend::Blockfrost), "bf-{network}");
        assert!(router.supports_network(Backend::Blockfrost, "vector-testnet"));
        assert!(router.supports_network(Backend::SubmitApi, "cardano-mainnet"));
        assert!(!router.supports_network(Backend::SubmitApi, "vector-testnet"));
        assert_eq!(
            router.backend_template(Backend::SubmitApi),
            "submit-{network}"
        );
    }

    #[test]
    fn routing_config_deserializes_backend_support() {
        let cfg: RoutingConfig = toml::from_str(
            r#"
default_backend = "blockfrost"

[backends.blockfrost]
template = "blockfrost-{network}:3000"

[backends.dolos]
template = "dolos-{network}:50051"
supported_networks = ["cardano-mainnet"]

[backends.submitapi]
template = "submit-{network}:8090"
supported_networks = ["cardano-mainnet", "cardano-preview"]

[[routes]]
path = "/blocks/{hash}"
backend = "dolos"
"#,
        )
        .unwrap();

        let router = cfg.build_router().unwrap();
        assert!(router.supports_network(Backend::Dolos, "cardano-mainnet"));
        assert!(!router.supports_network(Backend::Dolos, "cardano-preprod"));
    }
}
