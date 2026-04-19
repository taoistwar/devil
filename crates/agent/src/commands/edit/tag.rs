//! /tag 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct TagCommand;
impl TagCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TagCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for TagCommand {
    fn name(&self) -> &str {
        "tag"
    }
    fn description(&self) -> &str {
        "标签管理"
    }
    fn usage(&self) -> &str {
        "/tag [name]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "标签",
            serde_json::json!({"action": "tag", "args": args, "session_id": ctx.session_id}),
        )
    }
}
