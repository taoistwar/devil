//! /clear 命令 - 清除对话

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /clear 命令
pub struct ClearCommand;

impl ClearCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClearCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }

    fn description(&self) -> &str {
        "清除当前对话"
    }

    fn usage(&self) -> &str {
        "/clear"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!(
            "Clearing conversation history for session: {}",
            ctx.session_id
        );

        CommandResult::success_with_data(
            "对话已清除",
            serde_json::json!({
                "action": "clear",
                "session_id": ctx.session_id,
                "status": "cleared"
            }),
        )
    }
}
