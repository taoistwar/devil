//! Command dispatcher - routes CLI commands to handlers
//!
//! Provides structured command dispatch with proper error handling and exit codes

use crate::cli::error::CliError;
use crate::cli::{run_once, run_repl, run_web, show_config, APP_NAME, VERSION};
use anyhow::Result;
use std::process::ExitCode;

pub trait Command: Send + Sync {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    #[allow(dead_code)]
    fn description(&self) -> &str;
    #[allow(dead_code)]
    fn execute(&self, args: &[&str]) -> Result<()>;
}

pub struct Dispatcher {
    #[allow(dead_code)]
    commands: Vec<Box<dyn Command>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn register<C: Command + 'static>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }

    pub fn dispatch(&self, args: &[String]) -> Result<ExitCode, CliError> {
        if args.is_empty() {
            self.print_help()?;
            return Ok(ExitCode::SUCCESS);
        }

        let first_arg = args[0].as_str();

        // Handle slash commands (e.g., /help, /compact)
        if first_arg.starts_with('/') {
            return self.dispatch_slash_command(first_arg, &args[1..]);
        }

        match first_arg {
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
            "web" => {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| CliError::InitError(format!("Failed to create runtime: {}", e)))?;
                if let Err(e) = rt.block_on(run_web(&args[1..])) {
                    tracing::error!("Web server failed: {}", e);
                    return Err(CliError::WebError(e.to_string()));
                }
                Ok(ExitCode::SUCCESS)
            }
            unknown => Err(CliError::InvalidCommand(unknown.to_string())),
        }
    }

    fn dispatch_slash_command(&self, cmd_name: &str, args: &[String]) -> Result<ExitCode, CliError> {
        let name = cmd_name.trim_start_matches('/');
        
        let ctx = devil_agent_core::commands::CommandContext::default();
        let cmd_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::InitError(format!("Failed to create runtime: {}", e)))?;
        
        let result = rt.block_on(async {
            let registry = devil_agent_core::commands::global_registry();
            registry.execute(name, &ctx, &cmd_args).await
        });
        
        match result {
            Some(result) => {
                if result.success {
                    if let Some(output) = result.output {
                        println!("{}", output);
                    }
                    Ok(ExitCode::SUCCESS)
                } else {
                    if let Some(error) = result.error {
                        eprintln!("Error: {}", error);
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
            None => {
                Err(CliError::InvalidCommand(format!("/{}", name)))
            }
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
        println!("Slash Commands (inside REPL):");
        println!("  /help [cmd]     Show help for a command");
        println!("  /compact         Manually compact context");
        println!("  /model [name]    Switch AI model");
        println!("  /clear           Clear conversation");
        println!("  /plan            Enter plan mode");
        println!("  /review          Code review mode");
        println!("  ... (95+ more commands available)");
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
