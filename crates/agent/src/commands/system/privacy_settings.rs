//! /privacy-settings 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PrivacySettingsCommand;
impl PrivacySettingsCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PrivacySettingsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for PrivacySettingsCommand {
    fn name(&self) -> &str {
        "privacy-settings"
    }
    fn description(&self) -> &str {
        "隐私设置"
    }
    fn usage(&self) -> &str {
        "/privacy-settings"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "隐私设置",
            serde_json::json!({"action": "privacy-settings", "session_id": ctx.session_id}),
        )
    }
}
