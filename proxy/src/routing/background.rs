use std::{error::Error, fs, sync::Arc};

use async_trait::async_trait;
use notify::{Event, PollWatcher, RecursiveMode, Watcher};
use pingora::{
    server::ShutdownWatch,
    services::{background::BackgroundService, ServiceReadyNotifier},
};
use tracing::{error, info, warn};

use crate::State;

use super::{RoutingConfig, ROUTER};

pub struct RoutingBackgroundService {
    state: Arc<State>,
    config_path: Arc<std::path::PathBuf>,
    poll_interval: std::time::Duration,
}

impl RoutingBackgroundService {
    pub fn new(
        state: Arc<State>,
        config_path: Arc<std::path::PathBuf>,
        poll_interval: std::time::Duration,
    ) -> Self {
        Self {
            state,
            config_path,
            poll_interval,
        }
    }

    async fn update_router(&self) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string(&*self.config_path)?;
        let cfg: RoutingConfig = toml::from_str(&contents)?;
        let router = cfg.build_router()?;
        ROUTER.store(std::sync::Arc::new(router));
        Ok(())
    }
}

#[async_trait]
impl BackgroundService for RoutingBackgroundService {
    async fn start_with_ready_notifier(
        &self,
        mut shutdown: ShutdownWatch,
        ready_notifier: ServiceReadyNotifier,
    ) {
        if let Err(err) = self.update_router().await {
            error!(error = err.to_string(), "error to update routing");
            return;
        }

        self.state.set_routing_ready();
        ready_notifier.notify_ready();

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1);

        let watcher_config = notify::Config::default()
            .with_compare_contents(true)
            .with_poll_interval(self.poll_interval);

        let watcher_result = PollWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            watcher_config,
        );

        if let Err(err) = watcher_result {
            error!(error = err.to_string(), "error to watcher routing");
            return;
        }

        let mut watcher = watcher_result.unwrap();
        if let Err(err) = watcher.watch(&self.config_path, RecursiveMode::Recursive) {
            error!(error = err.to_string(), "error to watcher routing");
            return;
        }

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    info!("routing: shutdown requested");
                    break;
                }
                event = rx.recv() => {
                    if event.is_some() {
                        match self.update_router().await {
                            Ok(_) => info!("routing modified"),
                            Err(err) => warn!(error = err.to_string(), "invalid routing reload"),
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
