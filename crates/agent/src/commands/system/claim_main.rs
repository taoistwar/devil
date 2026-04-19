//! /claim-main 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ClaimMainCommand;
impl ClaimMainCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ClaimMainCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ClaimMainCommand {
    fn name(&self) -> &str {
        "claim-main"
    }
    fn description(&self) -> &str {
        "声明主会话"
    }
    fn usage(&self) -> &str {
        "/claim-main"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "主会话已声明",
            serde_json::json!({"action": "claim-main", "session_id": ctx.session_id}),
        )
    }
}
