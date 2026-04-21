//! /history 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct HistoryCommand;
impl HistoryCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HistoryCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for HistoryCommand {
    fn name(&self) -> &str {
        "history"
    }
    fn description(&self) -> &str {
        "历史记录"
    }
    fn usage(&self) -> &str {
        "/history [limit]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "历史记录",
            serde_json::json!({"action": "history", "args": args, "session_id": ctx.session_id}),
        )
    }
}
