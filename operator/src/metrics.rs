use serde::{Deserialize, Deserializer};
use std::sync::Arc;

use chrono::Utc;
use kube::{Resource, ResourceExt};
use prometheus::{opts, IntCounterVec, Registry};
use regex::Regex;
use tracing::{error, info, instrument};

use crate::{get_config, BlockfrostPort, Error, Network, State};

#[derive(Clone)]
pub struct Metrics {
    pub dcu: IntCounterVec,
    pub reconcile_failures: IntCounterVec,
    pub metrics_failures: IntCounterVec,
}

impl Default for Metrics {
    fn default() -> Self {
        let dcu = IntCounterVec::new(
            opts!("dmtr_consumed_dcus", "quantity of dcu consumed",),
            &["project", "service", "service_type", "tenancy"],
        )
        .unwrap();

        let reconcile_failures = IntCounterVec::new(
            opts!(
                "crd_controller_reconciliation_errors_total",
                "reconciliation errors",
            ),
            &["instance", "error"],
        )
        .unwrap();

        let metrics_failures = IntCounterVec::new(
            opts!(
                "metrics_controller_errors_total",
                "errors to calculation metrics",
            ),
            &["error"],
        )
        .unwrap();

        Metrics {
            dcu,
            reconcile_failures,
            metrics_failures,
        }
    }
}

impl Metrics {
    pub fn register(self, registry: &Registry) -> Result<Self, prometheus::Error> {
        registry.register(Box::new(self.reconcile_failures.clone()))?;
        registry.register(Box::new(self.metrics_failures.clone()))?;
        registry.register(Box::new(self.dcu.clone()))?;

        Ok(self)
    }

    pub fn reconcile_failure(&self, crd: &BlockfrostPort, e: &Error) {
        self.reconcile_failures
            .with_label_values(&[crd.name_any().as_ref(), e.metric_label().as_ref()])
            .inc()
    }

    pub fn metrics_failure(&self, e: &Error) {
        self.metrics_failures
            .with_label_values(&[e.metric_label().as_ref()])
            .inc()
    }

    pub fn count_dcu_consumed(&self, ns: &str, network: Network, dcu: f64) {
        let project = ns.split_once("prj-").unwrap().1;
        let service = format!("{}-{}", BlockfrostPort::kind(&()), network);
        let service_type = format!(
            "{}.{}",
            BlockfrostPort::plural(&()),
            BlockfrostPort::group(&())
        );
        let tenancy = "proxy";

        let dcu: u64 = dcu.ceil() as u64;

        self.dcu
            .with_label_values(&[project, &service, &service_type, tenancy])
            .inc_by(dcu);
    }
}

#[instrument("metrics collector run", skip_all)]
pub async fn run_metrics_collector(state: Arc<State>) {
    tokio::spawn(async move {
        info!("collecting metrics running");

        let config = get_config();
        let client = reqwest::Client::builder().build().unwrap();
        let regex = Regex::new(r"(.+)\.(.+)-.+").unwrap();
        let mut last_execution = Utc::now();

        loop {
            tokio::time::sleep(config.metrics_delay).await;

            let end = Utc::now();
            let start = (end - last_execution).num_seconds();

            last_execution = end;

            let query = format!(
                    "sum by (consumer) (increase(kong_http_requests_total{{service='blockfrost-v1-ingress-kong-proxy'}}[{start}s] @ {}))",
                    end.timestamp_millis() / 1000
                );

            let result = client
                .get(format!("{}/query?query={query}", config.prometheus_url))
                .send()
                .await;

            if let Err(err) = result {
                error!(error = err.to_string(), "error to make prometheus request");
                state
                    .metrics
                    .metrics_failure(&Error::HttpError(err.to_string()));
                continue;
            }

            let response = result.unwrap();
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                error!(status = status.to_string(), "request status code fail");
                state.metrics.metrics_failure(&Error::HttpError(format!(
                    "Prometheus request error. Status: {} Query: {}",
                    status, query
                )));
                continue;
            }

            let response = response.json::<PrometheusResponse>().await.unwrap();

            for result in response.data.result {
                if let Some(consumer) = result.metric.consumer {
                    let captures = regex.captures(&consumer).unwrap();

                    let namespace = captures.get(1).unwrap().as_str();
                    let network: Network = captures.get(2).unwrap().as_str().try_into().unwrap();

                    if result.value == 0.0 {
                        continue;
                    }

                    let dcu_per_request = match network {
                        Network::Mainnet => config.dcu_per_request_mainnet,
                        Network::Preprod => config.dcu_per_request_preprod,
                        Network::Preview => config.dcu_per_request_preview,
                        Network::Sanchonet => config.dcu_per_request_sanchonet,
                    };

                    let dcu = result.value * dcu_per_request;
                    state.metrics.count_dcu_consumed(namespace, network, dcu);
                }
            }
        }
    });
}

#[derive(Deserialize)]
struct PrometheusDataResultMetric {
    consumer: Option<String>,
}

#[derive(Deserialize)]
struct PrometheusDataResult {
    metric: PrometheusDataResultMetric,
    #[serde(deserialize_with = "deserialize_value")]
    value: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrometheusData {
    result: Vec<PrometheusDataResult>,
}

#[derive(Deserialize)]
struct PrometheusResponse {
    data: PrometheusData,
}

fn deserialize_value<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<serde_json::Value> = Deserialize::deserialize(deserializer)?;
    Ok(value.into_iter().as_slice()[1]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap())
}
