//! /remote-setup 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct RemoteSetupCommand;
impl RemoteSetupCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for RemoteSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for RemoteSetupCommand {
    fn name(&self) -> &str {
        "remote-setup"
    }
    fn description(&self) -> &str {
        "远程设置"
    }
    fn usage(&self) -> &str {
        "/remote-setup"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "远程设置",
            serde_json::json!({"action": "remote-setup", "session_id": ctx.session_id}),
        )
    }
}
