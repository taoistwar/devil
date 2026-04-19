//! /oauth-refresh 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct OauthRefreshCommand;
impl OauthRefreshCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for OauthRefreshCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for OauthRefreshCommand {
    fn name(&self) -> &str {
        "oauth-refresh"
    }
    fn description(&self) -> &str {
        "OAuth 刷新"
    }
    fn usage(&self) -> &str {
        "/oauth-refresh"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "OAuth 刷新",
            serde_json::json!({"action": "oauth-refresh", "session_id": ctx.session_id}),
        )
    }
}
