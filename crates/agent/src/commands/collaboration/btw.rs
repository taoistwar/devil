//! /btw 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct BtwCommand;
impl BtwCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for BtwCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for BtwCommand {
    fn name(&self) -> &str {
        "btw"
    }
    fn description(&self) -> &str {
        "侧注"
    }
    fn usage(&self) -> &str {
        "/btw [text]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "侧注",
            serde_json::json!({"action": "btw", "args": args, "session_id": ctx.session_id}),
        )
    }
}
