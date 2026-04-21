//! /sandbox-toggle 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct SandboxToggleCommand;
impl SandboxToggleCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SandboxToggleCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for SandboxToggleCommand {
    fn name(&self) -> &str {
        "sandbox-toggle"
    }
    fn description(&self) -> &str {
        "沙箱切换"
    }
    fn usage(&self) -> &str {
        "/sandbox-toggle"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "沙箱切换",
            serde_json::json!({"action": "sandbox-toggle", "session_id": ctx.session_id}),
        )
    }
}
