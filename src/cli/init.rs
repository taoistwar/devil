//! CLI initialization and graceful shutdown
//!
//! Follows Claude Code's init.ts patterns for one-time initialization
//! and graceful shutdown handling.

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tokio::signal::unix::SignalKind;
use tokio::sync::RwLock;
use tracing::{error, info};

type CleanupFn = Box<dyn Fn() + Send + Sync>;

pub struct InitState {
    cleanup_handlers: RwLock<Vec<CleanupFn>>,
}

impl InitState {
    pub fn new() -> Self {
        Self {
            cleanup_handlers: RwLock::new(Vec::new()),
        }
    }

    pub async fn register_cleanup<F>(&self, f: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut handlers = self.cleanup_handlers.write().await;
        handlers.push(Box::new(f));
    }

    pub async fn shutdown(&self) {
        info!("Initiating graceful shutdown...");
        let handlers = self.cleanup_handlers.read().await;
        for handler in handlers.iter() {
            handler();
        }
        info!("Shutdown complete.");
    }
}

impl Default for InitState {
    fn default() -> Self {
        Self::new()
    }
}

static INIT_STATE: std::sync::OnceLock<Arc<InitState>> = std::sync::OnceLock::new();

pub fn get_init_state() -> &'static Arc<InitState> {
    INIT_STATE.get_or_init(|| Arc::new(InitState::new()))
}

pub async fn init() -> Result<()> {
    let state = get_init_state();
    state
        .register_cleanup(|| {
            info!("Cleanup: saving session state");
        })
        .await;
    state
        .register_cleanup(|| {
            info!("Cleanup: flushing telemetry");
        })
        .await;

    Ok(())
}

pub async fn wait_for_shutdown_signal() {
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received SIGINT (Ctrl+C)");
        }
        _ = async {
            match signal::unix::signal(SignalKind::terminate()) {
                Ok(mut sig) => {
                    let _ = sig.recv().await;
                }
                Err(e) => {
                    error!("Failed to set up SIGTERM handler: {}", e);
                }
            }
        } => {
            info!("Received SIGTERM");
        }
    }
}

pub async fn run_with_graceful_shutdown<F, Fut>(future: F) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let state = get_init_state();

    tokio::select! {
        result = future() => {
            if let Err(e) = result {
                error!("Task failed: {}", e);
            }
        }
        _ = wait_for_shutdown_signal() => {
            info!("Shutdown signal received");
        }
    }

    state.shutdown().await;
    Ok(())
}
