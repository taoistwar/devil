//! /debug-tool-call 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct DebugToolCallCommand;
impl DebugToolCallCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for DebugToolCallCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for DebugToolCallCommand {
    fn name(&self) -> &str {
        "debug-tool-call"
    }
    fn description(&self) -> &str {
        "调试工具调用"
    }
    fn usage(&self) -> &str {
        "/debug-tool-call"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "调试模式",
            serde_json::json!({"action": "debug-tool-call", "session_id": ctx.session_id}),
        )
    }
}
