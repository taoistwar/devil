//! /passes 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PassesCommand;
impl PassesCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PassesCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for PassesCommand {
    fn name(&self) -> &str {
        "passes"
    }
    fn description(&self) -> &str {
        "代码改进"
    }
    fn usage(&self) -> &str {
        "/passes"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "代码改进",
            serde_json::json!({"action": "passes", "session_id": ctx.session_id}),
        )
    }
}
