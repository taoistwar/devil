//! /fast 命令 - 快速模式

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct FastCommand;

impl FastCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FastCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for FastCommand {
    fn name(&self) -> &str {
        "fast"
    }

    fn description(&self) -> &str {
        "切换快速模式"
    }

    fn usage(&self) -> &str {
        "/fast"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Fast mode requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "快速模式已启用",
            serde_json::json!({
                "action": "fast",
                "mode": "fast",
                "session_id": ctx.session_id
            }),
        )
    }
}
