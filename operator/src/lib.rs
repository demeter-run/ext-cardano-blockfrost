use std::fmt::{self, Display, Formatter};

use prometheus::Registry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kube Error: {0}")]
    KubeError(#[source] kube::Error),

    #[error("Deserialize Error: {0}")]
    DeserializeError(#[source] serde_json::Error),

    #[error("Parse Network error: {0}")]
    ParseNetworkError(String),

    #[error("Http Request error: {0}")]
    HttpError(String),

    #[error("Config Error: {0}")]
    ConfigError(String),
}

impl Error {
    pub fn metric_label(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::DeserializeError(value)
    }
}

impl From<kube::Error> for Error {
    fn from(value: kube::Error) -> Self {
        Error::KubeError(value)
    }
}

#[derive(Clone)]
pub struct State {
    registry: Registry,
    pub metrics: Metrics,
}
impl State {
    pub fn new() -> Self {
        let registry = Registry::default();
        let metrics = Metrics::default().register(&registry).unwrap();
        Self { registry, metrics }
    }

    pub fn metrics_collected(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub enum Network {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "preprod")]
    Preprod,
    #[serde(rename = "preview")]
    Preview,
    #[serde(rename = "sanchonet")]
    Sanchonet,
}
impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Preprod => write!(f, "preprod"),
            Network::Preview => write!(f, "preview"),
            Network::Sanchonet => write!(f, "sanchonet"),
        }
    }
}
impl TryFrom<&str> for Network {
    type Error = Error;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        match value {
            "mainnet" => Ok(Network::Mainnet),
            "preprod" => Ok(Network::Preprod),
            "preview" => Ok(Network::Preview),
            "sanchonet" => Ok(Network::Sanchonet),
            network => Err(Error::ParseNetworkError(network.into())),
        }
    }
}

pub use k8s_openapi;
pub use kube;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod controller;
pub use crate::controller::*;

pub mod metrics;
pub use metrics::*;

mod config;
pub use config::*;

mod utils;
pub use utils::*;
