//! /effort 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct EffortCommand;
impl EffortCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for EffortCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for EffortCommand {
    fn name(&self) -> &str {
        "effort"
    }
    fn description(&self) -> &str {
        "估算工作量"
    }
    fn usage(&self) -> &str {
        "/effort"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "工作量估算",
            serde_json::json!({"action": "effort", "session_id": ctx.session_id}),
        )
    }
}
