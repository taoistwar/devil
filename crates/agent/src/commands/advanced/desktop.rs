//! /desktop 命令 - 桌面应用

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct DesktopCommand;

impl DesktopCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DesktopCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for DesktopCommand {
    fn name(&self) -> &str {
        "desktop"
    }

    fn description(&self) -> &str {
        "桌面应用"
    }

    fn usage(&self) -> &str {
        "/desktop"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Desktop app requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "正在打开桌面应用...",
            serde_json::json!({
                "action": "desktop",
                "session_id": ctx.session_id
            }),
        )
    }
}
