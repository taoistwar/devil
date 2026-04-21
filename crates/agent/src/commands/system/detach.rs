//! /detach 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct DetachCommand;
impl DetachCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for DetachCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for DetachCommand {
    fn name(&self) -> &str {
        "detach"
    }
    fn description(&self) -> &str {
        "分离会话"
    }
    fn usage(&self) -> &str {
        "/detach"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "会话已分离",
            serde_json::json!({"action": "detach", "session_id": ctx.session_id}),
        )
    }
}
