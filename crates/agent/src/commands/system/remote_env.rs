//! /remote-env 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct RemoteEnvCommand;
impl RemoteEnvCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for RemoteEnvCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for RemoteEnvCommand {
    fn name(&self) -> &str {
        "remote-env"
    }
    fn description(&self) -> &str {
        "远程环境"
    }
    fn usage(&self) -> &str {
        "/remote-env"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "远程环境",
            serde_json::json!({"action": "remote-env", "session_id": ctx.session_id}),
        )
    }
}
