//! /poor 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PoorCommand;
impl PoorCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PoorCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for PoorCommand {
    fn name(&self) -> &str {
        "poor"
    }
    fn description(&self) -> &str {
        "反馈差的体验"
    }
    fn usage(&self) -> &str {
        "/poor [comment]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "感谢反馈, 我们会改进",
            serde_json::json!({"action": "poor", "args": args, "session_id": ctx.session_id}),
        )
    }
}
