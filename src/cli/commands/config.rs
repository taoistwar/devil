//! Config command implementation

use super::super::dispatcher::Command;
use crate::cli::show_config;
use anyhow::Result;

pub struct ConfigCommand;

impl ConfigCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ConfigCommand {
    fn name(&self) -> &str {
        "config"
    }

    fn description(&self) -> &str {
        "Show configuration"
    }

    fn execute(&self, args: &[&str]) -> Result<()> {
        if args.len() > 1 {
            match args[1] {
                "show" | "get" | "set" => {
                    // Config subcommands - placeholder for now
                    println!("Config subcommand: {}", args[1]);
                }
                _ => {
                    anyhow::bail!("Unknown config subcommand: {}", args[1]);
                }
            }
        }
        show_config()?;
        Ok(())
    }
}

impl Default for ConfigCommand {
    fn default() -> Self {
        Self::new()
    }
}
