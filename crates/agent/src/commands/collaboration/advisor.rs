//! /advisor 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct AdvisorCommand;
impl AdvisorCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for AdvisorCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for AdvisorCommand {
    fn name(&self) -> &str {
        "advisor"
    }
    fn description(&self) -> &str {
        "顾问模式"
    }
    fn usage(&self) -> &str {
        "/advisor"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "顾问模式",
            serde_json::json!({"action": "advisor", "session_id": ctx.session_id}),
        )
    }
}
