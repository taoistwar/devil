//! /theme 命令 - 主题切换

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /theme 命令
pub struct ThemeCommand;

impl ThemeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThemeCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ThemeCommand {
    fn name(&self) -> &str {
        "theme"
    }

    fn description(&self) -> &str {
        "切换主题"
    }

    fn usage(&self) -> &str {
        "/theme [theme-name]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let theme = args
            .first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "dark".to_string());

        tracing::info!(
            "Theme switch requested: {} for session: {}",
            theme,
            ctx.session_id
        );

        CommandResult::success_with_data(
            format!("主题已切换到: {}", theme),
            serde_json::json!({
                "action": "theme",
                "theme": theme,
                "session_id": ctx.session_id
            }),
        )
    }
}
