//! /ide 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct IdeCommand;
impl IdeCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for IdeCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for IdeCommand {
    fn name(&self) -> &str {
        "ide"
    }
    fn description(&self) -> &str {
        "IDE 设置"
    }
    fn usage(&self) -> &str {
        "/ide [subcommand]"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "IDE 设置",
            serde_json::json!({"action": "ide", "session_id": ctx.session_id}),
        )
    }
}
