use std::error::Error;
use std::{fs, sync::Arc};

use async_trait::async_trait;
use notify::{Event, PollWatcher, RecursiveMode, Watcher};
use pingora::{
    server::ShutdownWatch,
    services::{background::BackgroundService, ServiceReadyNotifier},
};
use serde_json::Value;
use tracing::{error, info, warn};

use crate::{config::Config, State, Tier};

pub struct TierBackgroundService {
    state: Arc<State>,
    config: Arc<Config>,
}
impl TierBackgroundService {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        Self { state, config }
    }

    async fn update_tiers(&self) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string(&self.config.proxy_tiers_path)?;

        let value: Value = toml::from_str(&contents)?;
        let tiers_value: Option<&Value> = value.get("tiers");
        if tiers_value.is_none() {
            warn!("tiers not configured on toml");
            return Ok(());
        }

        let tiers = serde_json::from_value::<Vec<Tier>>(tiers_value.unwrap().to_owned())?;

        *self.state.tiers.write().await = tiers
            .into_iter()
            .map(|tier| (tier.name.clone(), tier))
            .collect();

        self.state.limiter.write().await.clear();

        Ok(())
    }
}

#[async_trait]
impl BackgroundService for TierBackgroundService {
    async fn start_with_ready_notifier(
        &self,
        mut shutdown: ShutdownWatch,
        ready_notifier: ServiceReadyNotifier,
    ) {
        if let Err(err) = self.update_tiers().await {
            error!(error = err.to_string(), "error to update tiers");
            return;
        }

        self.state.set_tiers_ready();
        ready_notifier.notify_ready();

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1);

        let watcher_config = notify::Config::default()
            .with_compare_contents(true)
            .with_poll_interval(self.config.proxy_tiers_poll_interval);

        let watcher_result = PollWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            watcher_config,
        );
        if let Err(err) = watcher_result {
            error!(error = err.to_string(), "error to watcher tier");
            return;
        }

        let mut watcher = watcher_result.unwrap();
        let watcher_result = watcher.watch(&self.config.proxy_tiers_path, RecursiveMode::Recursive);
        if let Err(err) = watcher_result {
            error!(error = err.to_string(), "error to watcher tier");
            return;
        }

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    info!("tiers: shutdown requested");
                    break;
                }
                result = rx.recv() => {
                    if result.is_some() {
                        if let Err(err) = self.update_tiers().await {
                            error!(error = err.to_string(), "error to update tiers");
                            continue;
                        }
                        info!("tiers modified");
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
