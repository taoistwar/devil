//! /teleport 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct TeleportCommand;
impl TeleportCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TeleportCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for TeleportCommand {
    fn name(&self) -> &str {
        "teleport"
    }
    fn description(&self) -> &str {
        "远程切换"
    }
    fn usage(&self) -> &str {
        "/teleport [target]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "远程切换",
            serde_json::json!({"action": "teleport", "args": args, "session_id": ctx.session_id}),
        )
    }
}
