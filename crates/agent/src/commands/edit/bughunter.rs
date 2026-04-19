//! /bughunter 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct BughunterCommand;
impl BughunterCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for BughunterCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for BughunterCommand {
    fn name(&self) -> &str {
        "bughunter"
    }
    fn description(&self) -> &str {
        "Bug 追踪"
    }
    fn usage(&self) -> &str {
        "/bughunter"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "Bug 追踪",
            serde_json::json!({"action": "bughunter", "session_id": ctx.session_id}),
        )
    }
}
