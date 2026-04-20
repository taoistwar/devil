//! Web server lifecycle management

use crate::web::handler::{create_router, AppState};
use crate::web::ApiError;
use axum::serve;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::Mutex;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_HOST: &str = "127.0.0.1";

/// Web server instance
pub struct WebServer {
    host: String,
    port: u16,
}

impl WebServer {
    /// Create a new web server with default settings
    pub fn new() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }

    /// Create a new web server with custom host and port
    pub fn with_config(host: String, port: u16) -> Self {
        Self { host, port }
    }

    /// Set the host address
    pub fn with_host(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Get the socket address
    fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
    }

    /// Start the server and wait for shutdown
    pub async fn serve(self) -> Result<(), ApiError> {
        let state = Arc::new(Mutex::new(AppState::new()));

        let addr = self.socket_addr();
        let listener = TcpListener::bind(addr).await.map_err(|e| ApiError::PortInUse(e.to_string()))?;
        
        tracing::info!("Web server starting on {}", addr);

        serve(listener, create_router(state))
            .with_graceful_shutdown(async {
                signal::ctrl_c().await.ok();
            })
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        tracing::info!("Web server stopped");
        Ok(())
    }

    /// Run the server with signal handling for Ctrl+C
    pub async fn serve_with_signals(self) -> Result<(), ApiError> {
        self.serve().await
    }
}

impl Default for WebServer {
    fn default() -> Self {
        Self::new()
    }
}
