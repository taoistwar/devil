//! /upgrade 命令 - 自动更新

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct UpgradeCommand;

impl UpgradeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UpgradeCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for UpgradeCommand {
    fn name(&self) -> &str {
        "upgrade"
    }

    fn description(&self) -> &str {
        "自动更新"
    }

    fn usage(&self) -> &str {
        "/upgrade"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Upgrade requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "正在检查更新...",
            serde_json::json!({
                "action": "upgrade",
                "session_id": ctx.session_id,
                "current_version": "0.1.0"
            }),
        )
    }
}
