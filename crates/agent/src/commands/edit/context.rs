//! /context 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ContextCommand;
impl ContextCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ContextCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ContextCommand {
    fn name(&self) -> &str {
        "context"
    }
    fn description(&self) -> &str {
        "上下文管理"
    }
    fn usage(&self) -> &str {
        "/context [subcommand]"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "上下文",
            serde_json::json!({"action": "context", "session_id": ctx.session_id}),
        )
    }
}
