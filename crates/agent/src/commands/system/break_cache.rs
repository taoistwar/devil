//! /break-cache 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct BreakCacheCommand;
impl BreakCacheCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for BreakCacheCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for BreakCacheCommand {
    fn name(&self) -> &str {
        "break-cache"
    }
    fn description(&self) -> &str {
        "清除缓存"
    }
    fn usage(&self) -> &str {
        "/break-cache"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "缓存已清除",
            serde_json::json!({"action": "break-cache", "session_id": ctx.session_id}),
        )
    }
}
