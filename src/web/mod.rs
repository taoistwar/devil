//! Web server module
//!
//! Provides HTTP server functionality for the Devil CLI

pub mod error;
pub mod handler;
pub mod server;

pub use error::ApiError;
pub use handler::{chat_handler, health_handler, shutdown_handler, ChatRequest, ChatResponse, HealthResponse};
pub use server::WebServer;
