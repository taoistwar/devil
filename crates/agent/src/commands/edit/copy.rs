//! /copy 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct CopyCommand;
impl CopyCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for CopyCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for CopyCommand {
    fn name(&self) -> &str {
        "copy"
    }
    fn description(&self) -> &str {
        "复制内容"
    }
    fn usage(&self) -> &str {
        "/copy [text]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "内容已复制",
            serde_json::json!({"action": "copy", "args": args, "session_id": ctx.session_id}),
        )
    }
}
