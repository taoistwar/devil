//! /backfill-sessions 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct BackfillSessionsCommand;
impl BackfillSessionsCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for BackfillSessionsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for BackfillSessionsCommand {
    fn name(&self) -> &str {
        "backfill-sessions"
    }
    fn description(&self) -> &str {
        "会话填充"
    }
    fn usage(&self) -> &str {
        "/backfill-sessions"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "会话填充",
            serde_json::json!({"action": "backfill-sessions", "session_id": ctx.session_id}),
        )
    }
}
