//! CLI error types
//!
//! Provides specific error types with standardized exit codes for production use:
//! - 0: Success
//! - 1: General error
//! - 2: Configuration error
//! - 3: Missing API key
//! - 4: Invalid arguments
//! - 5: Network/connection error

use std::process::ExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error(
        "Missing API key: Set DEVIL_API_KEY environment variable or add api_key to config file"
    )]
    MissingApiKey,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Initialization failed: {0}")]
    InitError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Tool execution failed: {0}")]
    ToolError(String),
}

impl CliError {
    /// Get the exit code for this error
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::InvalidCommand(_) => ExitCode::from(4),
            CliError::MissingArgument(_) => ExitCode::from(4),
            CliError::ConfigError(_) => ExitCode::from(2),
            CliError::MissingApiKey => ExitCode::from(3),
            CliError::IoError(_) => ExitCode::from(1),
            CliError::InitError(_) => ExitCode::from(1),
            CliError::NetworkError(_) => ExitCode::from(5),
            CliError::ToolError(_) => ExitCode::from(1),
        }
    }
}
