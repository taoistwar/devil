//! Web server request handlers

use crate::web::error::ApiError;
use axum::{
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use devil_agent_core::{Agent, AgentConfig, UserMessage, AgentMessage as Message};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Chat request payload
#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    #[serde(default)]
    pub stream: bool,
}

/// Chat response payload
#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub success: bool,
    pub turns: usize,
    pub terminal_reason: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Shutdown response
#[derive(Debug, Clone, Serialize)]
pub struct ShutdownResponse {
    pub message: String,
}

/// Shared application state (placeholder for future use)
pub struct AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check handler
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Chat handler - uses Agent::run_once for processing
pub async fn chat_handler(
    _headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ApiError> {
    if req.prompt.is_empty() {
        return Err(ApiError::BadRequest("Prompt cannot be empty".to_string()));
    }

    if req.prompt.len() > 100000 {
        return Err(ApiError::BadRequest("Prompt too long (max 100000 chars)".to_string()));
    }

    // Create agent config
    let agent_config = AgentConfig {
        name: "devil-web".to_string(),
        model: "claude-sonnet-4-20250514".to_string(),
        system_prompt: get_system_prompt(),
        max_turns: 10,
        max_context_tokens: 200000,
        ..Default::default()
    };

    // Create and initialize agent
    let agent = Agent::new(agent_config).map_err(|e| ApiError::AgentError(e.to_string()))?;
    agent.initialize().await.map_err(|e| ApiError::AgentError(e.to_string()))?;

    // Create user message
    let user_message = Message::User(UserMessage::text(&req.prompt));

    // Run agent
    let result = agent.run_once(user_message).await.map_err(|e| ApiError::AgentError(e.to_string()))?;

    // Extract response text
    let response_text = result
        .messages
        .iter()
        .filter_map(|msg| {
            if let Message::Assistant(asm) = msg {
                Some(asm.text_content())
            } else {
                None
            }
        })
        .last()
        .unwrap_or_default();

    // Shutdown agent
    agent.shutdown().await.map_err(|e| ApiError::AgentError(e.to_string()))?;

    Ok(Json(ChatResponse {
        response: response_text,
        success: true,
        turns: result.turn_count,
        terminal_reason: format!("{:?}", result.terminal.reason),
    }))
}

fn get_system_prompt() -> String {
    r#"You are Devil Agent, an AI-powered development assistant. Keep responses concise."#.to_string()
}

/// Shutdown handler - returns success immediately
/// The actual shutdown is triggered by closing this connection
pub async fn shutdown_handler() -> Json<ShutdownResponse> {
    Json(ShutdownResponse {
        message: "Shutdown signal sent".to_string(),
    })
}

/// Create the router with all routes
pub fn create_router(_state: Arc<Mutex<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/chat", post(chat_handler))
        .route("/shutdown", get(shutdown_handler))
}
