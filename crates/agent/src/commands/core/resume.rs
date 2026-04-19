//! /resume 命令 - 恢复会话

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /resume 命令
pub struct ResumeCommand;

impl ResumeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResumeCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ResumeCommand {
    fn name(&self) -> &str {
        "resume"
    }

    fn description(&self) -> &str {
        "恢复之前的会话"
    }

    fn usage(&self) -> &str {
        "/resume [session-id]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let session_id = args
            .first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| ctx.session_id.clone());

        tracing::info!("Resuming session: {}", session_id);

        CommandResult::success_with_data(
            format!("正在恢复会话: {}", session_id),
            serde_json::json!({
                "action": "resume",
                "session_id": session_id,
                "status": "resuming"
            }),
        )
    }
}
