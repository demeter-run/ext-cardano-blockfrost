use serde::Deserialize;

use super::router::Router;
use super::trie::RouteTrie;
use super::{Backend, RoutingError};

#[derive(Debug, Deserialize, Clone)]
pub struct RoutingConfig {
    #[serde(default = "default_backend")]
    pub default_backend: String,

    #[serde(default)]
    pub backend_templates: BackendTemplateConfig,

    #[serde(default)]
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub backend: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendTemplateConfig {
    pub blockfrost: String,
    pub dolos: String,
    pub submitapi: String,
}

fn default_backend() -> String {
    "blockfrost".to_string()
}

impl Default for BackendTemplateConfig {
    fn default() -> Self {
        Self {
            blockfrost: "blockfrost-{network}:3000".to_string(),
            dolos: "internal-{network}-minibf:50051".to_string(),
            submitapi: "submitapi-{network}:8090".to_string(),
        }
    }
}

impl RoutingConfig {
    pub fn build_router(&self) -> Result<Router, RoutingError> {
        let default_backend = Backend::from_str(&self.default_backend)?;
        let mut trie = RouteTrie::new();

        for route in &self.routes {
            let backend = Backend::from_str(&route.backend)?;
            trie.insert(&route.path, backend)?;
        }

        Ok(Router::new(
            default_backend,
            trie,
            self.backend_templates.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backend_applied() {
        let cfg = RoutingConfig {
            default_backend: "blockfrost".into(),
            backend_templates: BackendTemplateConfig::default(),
            routes: vec![],
        };
        let router = cfg.build_router().unwrap();
        assert_eq!(router.resolve("/unknown"), Backend::Blockfrost);
    }

    #[test]
    fn dolos_route_resolves() {
        let cfg = RoutingConfig {
            default_backend: "blockfrost".into(),
            backend_templates: BackendTemplateConfig::default(),
            routes: vec![RouteConfig {
                path: "/blocks/:hash".into(),
                backend: "dolos".into(),
            }],
        };
        let router = cfg.build_router().unwrap();
        assert_eq!(router.resolve("/blocks/abc"), Backend::Dolos);
    }

    #[test]
    fn template_config_passthrough() {
        let cfg = RoutingConfig {
            default_backend: "blockfrost".into(),
            backend_templates: BackendTemplateConfig {
                blockfrost: "bf-{network}".into(),
                dolos: "dolos-{network}".into(),
                submitapi: "submit-{network}".into(),
            },
            routes: vec![],
        };
        let router = cfg.build_router().unwrap();
        assert_eq!(router.backend_template(Backend::Blockfrost), "bf-{network}");
        assert_eq!(
            router.backend_template(Backend::SubmitApi),
            "submit-{network}"
        );
    }
}
