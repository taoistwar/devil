//! /compact 命令 - 手动压缩上下文

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /compact 命令
pub struct CompactCommand;

impl CompactCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompactCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for CompactCommand {
    fn name(&self) -> &str {
        "compact"
    }

    fn description(&self) -> &str {
        "手动压缩上下文"
    }

    fn usage(&self) -> &str {
        "/compact"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Manual context compaction requested");

        CommandResult::success_with_data(
            "上下文压缩已触发",
            serde_json::json!({
                "action": "compact",
                "session_id": ctx.session_id,
                "status": "triggered"
            }),
        )
    }
}
