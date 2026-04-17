//! Command dispatcher - routes CLI commands to handlers

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

    pub fn dispatch(&self, args: &[String]) -> Result<ExitCode> {
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
                    eprintln!("Error: Missing prompt argument");
                    eprintln!("Usage: {} run \"<prompt>\"", APP_NAME);
                    return Ok(ExitCode::from(1));
                }
                let prompt = args[1..].join(" ");
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(run_once(&prompt))?;
                Ok(ExitCode::SUCCESS)
            }
            "repl" => {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(run_repl())?;
                Ok(ExitCode::SUCCESS)
            }
            "config" => {
                show_config()?;
                Ok(ExitCode::SUCCESS)
            }
            unknown => {
                eprintln!("Unknown command: {}", unknown);
                self.print_help()?;
                Ok(ExitCode::from(1))
            }
        }
    }

    pub fn print_help(&self) -> Result<()> {
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
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::from(1));
    }
}
