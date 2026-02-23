use std::{error::Error, fs, sync::Arc};

use async_trait::async_trait;
use notify::{Event, PollWatcher, RecursiveMode, Watcher};
use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use tokio::runtime::{Handle, Runtime};
use tracing::{error, info, warn};

use super::{ROUTER, RoutingConfig};

pub struct RoutingBackgroundService {
    config_path: Arc<std::path::PathBuf>,
    poll_interval: std::time::Duration,
}

impl RoutingBackgroundService {
    pub fn new(config_path: Arc<std::path::PathBuf>, poll_interval: std::time::Duration) -> Self {
        Self { config_path, poll_interval }
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
    async fn start(&self, mut _shutdown: ShutdownWatch) {
        if let Err(err) = self.update_router().await {
            error!(error = err.to_string(), "error to update routing");
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1);

        let watcher_config = notify::Config::default()
            .with_compare_contents(true)
            .with_poll_interval(self.poll_interval);

        let watcher_result = PollWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    runtime_handle()
                        .block_on(async { tx.send(event).await })
                        .unwrap();
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
            if rx.recv().await.is_some() {
                match self.update_router().await {
                    Ok(_) => info!("routing modified"),
                    Err(err) => warn!(error = err.to_string(), "invalid routing reload"),
                }
            }
        }
    }
}

fn runtime_handle() -> Handle {
    match Handle::try_current() {
        Ok(h) => h,
        Err(_) => Runtime::new().unwrap().handle().clone(),
    }
}
