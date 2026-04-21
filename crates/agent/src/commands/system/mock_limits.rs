//! /mock-limits 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct MockLimitsCommand;
impl MockLimitsCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for MockLimitsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for MockLimitsCommand {
    fn name(&self) -> &str {
        "mock-limits"
    }
    fn description(&self) -> &str {
        "模拟限制"
    }
    fn usage(&self) -> &str {
        "/mock-limits"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "模拟限制已设置",
            serde_json::json!({"action": "mock-limits", "session_id": ctx.session_id}),
        )
    }
}
