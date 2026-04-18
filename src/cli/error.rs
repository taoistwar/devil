//! CLI error types

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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Initialization failed: {0}")]
    InitError(String),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::InvalidCommand(_) => ExitCode::from(1),
            CliError::MissingArgument(_) => ExitCode::from(1),
            CliError::ConfigError(_) => ExitCode::from(1),
            CliError::IoError(_) => ExitCode::from(1),
            CliError::InitError(_) => ExitCode::from(1),
        }
    }
}
