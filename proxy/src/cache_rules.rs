use std::error::Error;
use std::{fs, sync::Arc};

use async_trait::async_trait;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use tokio::runtime::{Handle, Runtime};
use tracing::{error, info, warn};

use crate::{config::Config, State};

#[derive(Debug, Clone, Deserialize)]
pub struct CacheRule {
    #[serde(deserialize_with = "deserialize_endpoint")]
    pub endpoint: Regex,
    pub duration_s: u64,
}
pub fn deserialize_endpoint<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Regex, D::Error> {
    let value: String = Deserialize::deserialize(deserializer)?;
    match Regex::new(value.as_str()) {
        Ok(regex) => Ok(regex),
        Err(_) => Err(<D::Error as serde::de::Error>::custom("Invalid regex")),
    }
}
impl CacheRule {
    pub fn matches(&self, uri: &str) -> bool {
        self.endpoint.is_match(uri)
    }
}

pub struct CacheRuleBackgroundService {
    state: Arc<State>,
    config: Arc<Config>,
}
impl CacheRuleBackgroundService {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        Self { state, config }
    }

    async fn update_cache_rules(&self) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string(&self.config.cache_rules_path)?;

        let value: Value = toml::from_str(&contents)?;
        let cache_rules_value: Option<&Value> = value.get("rules");

        if cache_rules_value.is_none() {
            warn!("cache rules not configured on toml");
            return Ok(());
        }

        let cache_rules =
            serde_json::from_value::<Vec<CacheRule>>(cache_rules_value.unwrap().to_owned())?;

        *self.state.cache_rules.write().await = cache_rules;

        Ok(())
    }
}

pub fn runtime_handle() -> Handle {
    match Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            let rt = Runtime::new().unwrap();
            rt.handle().clone()
        }
    }
}

#[async_trait]
impl BackgroundService for CacheRuleBackgroundService {
    async fn start(&self, mut _shutdown: ShutdownWatch) {
        if let Err(err) = self.update_cache_rules().await {
            error!(error = err.to_string(), "error to update cache_rules");
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1);

        let watcher_result = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                let event = result.unwrap();
                if event.kind.is_modify() {
                    runtime_handle().block_on(async {
                        tx.send(event).await.unwrap();
                    });
                }
            },
            notify::Config::default(),
        );
        if let Err(err) = watcher_result {
            error!(error = err.to_string(), "error to watcher cache_rule");
            return;
        }

        let mut watcher = watcher_result.unwrap();
        let watcher_result = watcher.watch(&self.config.cache_rules_path, RecursiveMode::Recursive);
        if let Err(err) = watcher_result {
            error!(error = err.to_string(), "error to watcher cache_rule");
            return;
        }

        loop {
            if rx.recv().await.is_some() {
                if let Err(err) = self.update_cache_rules().await {
                    error!(error = err.to_string(), "error to update cache_rules");
                    continue;
                }

                info!("cache_rules modified");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_deserialize() {
        let value = json!({
            "endpoint": "/cacheable.*",
            "duration_s": 42,
        });
        let cache_rule: CacheRule = serde_json::from_value(value).expect("Fail to deserialize");
        assert!(cache_rule.matches("/cacheable"));
        assert!(cache_rule.matches("/cacheable/subpath"));
        assert_eq!(cache_rule.duration_s, 42);
    }
}
