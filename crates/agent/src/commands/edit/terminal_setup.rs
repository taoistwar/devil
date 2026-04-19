//! /terminalSetup 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct TerminalSetupCommand;
impl TerminalSetupCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TerminalSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for TerminalSetupCommand {
    fn name(&self) -> &str {
        "terminalSetup"
    }
    fn description(&self) -> &str {
        "终端设置"
    }
    fn usage(&self) -> &str {
        "/terminalSetup"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "终端设置",
            serde_json::json!({"action": "terminalSetup", "session_id": ctx.session_id}),
        )
    }
}
