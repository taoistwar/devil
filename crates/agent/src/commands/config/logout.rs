//! /logout 命令 - 登出认证

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /logout 命令
pub struct LogoutCommand;

impl LogoutCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LogoutCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for LogoutCommand {
    fn name(&self) -> &str {
        "logout"
    }

    fn description(&self) -> &str {
        "登出认证"
    }

    fn usage(&self) -> &str {
        "/logout"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Logout requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "已登出",
            serde_json::json!({
                "action": "logout",
                "session_id": ctx.session_id,
                "status": "logged_out"
            }),
        )
    }
}
