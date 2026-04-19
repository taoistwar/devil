//! /install-github-app 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct InstallGithubAppCommand;
impl InstallGithubAppCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for InstallGithubAppCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for InstallGithubAppCommand {
    fn name(&self) -> &str {
        "install-github-app"
    }
    fn description(&self) -> &str {
        "安装 GitHub 应用"
    }
    fn usage(&self) -> &str {
        "/install-github-app"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "GitHub 应用安装",
            serde_json::json!({"action": "install-github-app", "session_id": ctx.session_id}),
        )
    }
}
