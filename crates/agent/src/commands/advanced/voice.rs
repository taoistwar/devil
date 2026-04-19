//! /voice 命令 - 语音输入模式

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct VoiceCommand;

impl VoiceCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VoiceCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for VoiceCommand {
    fn name(&self) -> &str {
        "voice"
    }

    fn description(&self) -> &str {
        "语音输入模式"
    }

    fn usage(&self) -> &str {
        "/voice"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Voice mode requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "语音输入模式已启用",
            serde_json::json!({
                "action": "voice",
                "mode": "voice",
                "session_id": ctx.session_id
            }),
        )
    }
}
