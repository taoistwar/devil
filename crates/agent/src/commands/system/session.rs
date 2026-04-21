//! /session 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct SessionCommand;
impl SessionCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SessionCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for SessionCommand {
    fn name(&self) -> &str {
        "session"
    }
    fn description(&self) -> &str {
        "会话管理"
    }
    fn usage(&self) -> &str {
        "/session [subcommand]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "会话管理",
            serde_json::json!({"action": "session", "args": args, "session_id": ctx.session_id}),
        )
    }
}
