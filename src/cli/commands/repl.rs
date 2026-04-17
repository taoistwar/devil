//! REPL command implementation

use super::super::dispatcher::Command;
use crate::cli::run_repl;
use anyhow::Result;

pub struct ReplCommand;

impl ReplCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ReplCommand {
    fn name(&self) -> &str {
        "repl"
    }

    fn description(&self) -> &str {
        "Enter interactive mode"
    }

    fn execute(&self, _args: &[&str]) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(run_repl())?;
        Ok(())
    }
}

impl Default for ReplCommand {
    fn default() -> Self {
        Self::new()
    }
}
