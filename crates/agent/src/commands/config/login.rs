//! /login 命令 - 登录认证

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /login 命令
pub struct LoginCommand;

impl LoginCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoginCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for LoginCommand {
    fn name(&self) -> &str {
        "login"
    }

    fn description(&self) -> &str {
        "登录认证"
    }

    fn usage(&self) -> &str {
        "/login"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Login requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "正在打开登录页面...",
            serde_json::json!({
                "action": "login",
                "session_id": ctx.session_id,
                "status": "opening_auth_page"
            }),
        )
    }
}
