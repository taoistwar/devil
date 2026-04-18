//! Command dispatcher - routes CLI commands to handlers
//!
//! Provides structured command dispatch with proper error handling and exit codes

use crate::cli::error::CliError;
use crate::cli::{run_once, run_repl, show_config, APP_NAME, VERSION};
use anyhow::Result;
use std::process::ExitCode;

pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &[&str]) -> Result<()>;
}

pub struct Dispatcher {
    commands: Vec<Box<dyn Command>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn register<C: Command + 'static>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }

    pub fn dispatch(&self, args: &[String]) -> Result<ExitCode, CliError> {
        if args.is_empty() {
            self.print_help()?;
            return Ok(ExitCode::SUCCESS);
        }

        match args[0].as_str() {
            "--version" | "-v" | "-V" => {
                println!("{} v{}", APP_NAME, VERSION);
                Ok(ExitCode::SUCCESS)
            }
            "--help" | "-h" | "help" => {
                self.print_help()?;
                Ok(ExitCode::SUCCESS)
            }
            "run" => {
                if args.len() < 2 {
                    return Err(CliError::MissingArgument(
                        "Usage: devil run \"<prompt>\"".to_string(),
                    ));
                }
                let prompt = args[1..].join(" ");
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| CliError::InitError(format!("Failed to create runtime: {}", e)))?;
                if let Err(e) = rt.block_on(run_once(&prompt)) {
                    tracing::error!("Task execution failed: {}", e);
                    return Err(CliError::ToolError(e.to_string()));
                }
                Ok(ExitCode::SUCCESS)
            }
            "repl" => {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| CliError::InitError(format!("Failed to create runtime: {}", e)))?;
                if let Err(e) = rt.block_on(run_repl()) {
                    tracing::error!("REPL execution failed: {}", e);
                    return Err(CliError::ToolError(e.to_string()));
                }
                Ok(ExitCode::SUCCESS)
            }
            "config" => {
                show_config().map_err(|e| CliError::ConfigError(e.to_string()))?;
                Ok(ExitCode::SUCCESS)
            }
            unknown => Err(CliError::InvalidCommand(unknown.to_string())),
        }
    }

    pub fn print_help(&self) -> Result<(), CliError> {
        println!("{} v{}", APP_NAME, VERSION);
        println!();
        println!("Usage: {} <command> [arguments]", APP_NAME);
        println!();
        println!("Commands:");
        println!("  run <prompt>    Execute a single task");
        println!("  repl            Enter interactive mode");
        println!("  config          Show configuration");
        println!("  version, -v     Show version number");
        println!("  help, -h        Show this help message");
        println!();
        println!("Examples:");
        println!("  {} run \"analyze project structure\"", APP_NAME);
        println!("  {} repl", APP_NAME);
        Ok(())
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_version_flag() {
        let dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(&["--version".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[test]
    fn test_dispatcher_unknown_command() {
        let dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(&["unknown".to_string()]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().exit_code(), ExitCode::from(4));
    }

    #[test]
    fn test_dispatcher_missing_run_arg() {
        let dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(&["run".to_string()]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().exit_code(), ExitCode::from(4));
    }
}
