//! Help command implementation

use super::super::dispatcher::Command;
use crate::cli::dispatcher::Dispatcher;
use anyhow::Result;

pub struct HelpCommand {
    dispatcher: Dispatcher,
}

impl HelpCommand {
    pub fn new(dispatcher: Dispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "Show help message"
    }

    fn execute(&self, _args: &[&str]) -> Result<()> {
        self.dispatcher.print_help()?;
        Ok(())
    }
}
