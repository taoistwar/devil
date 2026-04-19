//! /stats 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct StatsCommand;
impl StatsCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for StatsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for StatsCommand {
    fn name(&self) -> &str {
        "stats"
    }
    fn description(&self) -> &str {
        "统计信息"
    }
    fn usage(&self) -> &str {
        "/stats"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "统计",
            serde_json::json!({"action": "stats", "session_id": ctx.session_id}),
        )
    }
}
