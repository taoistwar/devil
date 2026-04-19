//! /thinkback 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ThinkbackCommand;
impl ThinkbackCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ThinkbackCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ThinkbackCommand {
    fn name(&self) -> &str {
        "thinkback"
    }
    fn description(&self) -> &str {
        "Thinkback 工具"
    }
    fn usage(&self) -> &str {
        "/thinkback"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "Thinkback",
            serde_json::json!({"action": "thinkback", "session_id": ctx.session_id}),
        )
    }
}
