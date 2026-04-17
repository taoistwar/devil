//! Run command implementation

use super::super::dispatcher::Command;
use crate::cli::run_once;
use anyhow::Result;

pub struct RunCommand;

impl RunCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for RunCommand {
    fn name(&self) -> &str {
        "run"
    }

    fn description(&self) -> &str {
        "Execute a single task"
    }

    fn execute(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            anyhow::bail!("Missing prompt argument. Usage: devil run \"<prompt>\"");
        }
        let prompt = args.join(" ");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(run_once(&prompt))?;
        Ok(())
    }
}

impl Default for RunCommand {
    fn default() -> Self {
        Self::new()
    }
}
