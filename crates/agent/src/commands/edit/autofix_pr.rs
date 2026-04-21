//! /autofix-pr 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct AutofixPrCommand;
impl AutofixPrCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for AutofixPrCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for AutofixPrCommand {
    fn name(&self) -> &str {
        "autofix-pr"
    }
    fn description(&self) -> &str {
        "自动修复 PR"
    }
    fn usage(&self) -> &str {
        "/autofix-pr"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "自动修复 PR",
            serde_json::json!({"action": "autofix-pr", "session_id": ctx.session_id}),
        )
    }
}
