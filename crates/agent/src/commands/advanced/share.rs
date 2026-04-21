//! /share 命令 - 分享对话

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ShareCommand;

impl ShareCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShareCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ShareCommand {
    fn name(&self) -> &str {
        "share"
    }

    fn description(&self) -> &str {
        "分享当前会话"
    }

    fn usage(&self) -> &str {
        "/share"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Share requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "正在生成分享链接...",
            serde_json::json!({
                "action": "share",
                "session_id": ctx.session_id,
                "share_url": format!("https://claude.ai/share/{}", ctx.session_id)
            }),
        )
    }
}
