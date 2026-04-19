//! Version command implementation

use super::super::dispatcher::Command;
use crate::cli::show_version;
use anyhow::Result;

#[allow(dead_code)]
pub struct VersionCommand;

#[allow(dead_code)]
impl VersionCommand {
    pub fn new() -> Self {
        Self
    }
}

#[allow(dead_code)]
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
