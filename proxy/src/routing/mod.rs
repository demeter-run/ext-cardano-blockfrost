use arc_swap::ArcSwap;

pub mod background;
mod error;
mod trie;
mod router;
mod config;

pub use error::RoutingError;
pub use router::Router;
#[allow(unused_imports)]
pub use config::{BackendTemplateConfig, RouteConfig, RoutingConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    Blockfrost,
    Dolos,
    SubmitApi,
}

impl Backend {
    pub fn from_str(value: &str) -> Result<Self, RoutingError> {
        match value {
            "blockfrost" => Ok(Self::Blockfrost),
            "dolos" => Ok(Self::Dolos),
            "submitapi" => Ok(Self::SubmitApi),
            other => Err(RoutingError::UnknownBackend(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Backend::Blockfrost => "blockfrost",
            Backend::Dolos => "dolos",
            Backend::SubmitApi => "submitapi",
        }
    }
}

/// Global router instance swapped on config reload
pub static ROUTER: once_cell::sync::Lazy<ArcSwap<Router>> =
    once_cell::sync::Lazy::new(|| ArcSwap::from_pointee(Router::default()));
