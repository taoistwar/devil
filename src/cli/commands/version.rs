//! Version command implementation

use super::super::dispatcher::Command;
use crate::cli::show_version;
use anyhow::Result;

pub struct VersionCommand;

impl VersionCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }

    fn description(&self) -> &str {
        "Show version information"
    }

    fn execute(&self, _args: &[&str]) -> Result<()> {
        show_version();
        Ok(())
    }
}

impl Default for VersionCommand {
    fn default() -> Self {
        Self::new()
    }
}
