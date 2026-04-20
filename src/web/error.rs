//! Web API error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// API error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Agent error: {0}")]
    AgentError(String),

    #[error("Port already in use: {0}")]
    PortInUse(String),

    #[error("Server shutting down")]
    ShuttingDown,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required".to_string()),
            ApiError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Invalid API key".to_string()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::AgentError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::PortInUse(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::ShuttingDown => (StatusCode::SERVICE_UNAVAILABLE, "Server shutting down".to_string()),
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
