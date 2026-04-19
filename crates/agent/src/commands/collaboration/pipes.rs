//! /pipes 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PipesCommand;
impl PipesCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PipesCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for PipesCommand {
    fn name(&self) -> &str {
        "pipes"
    }
    fn description(&self) -> &str {
        "管道管理"
    }
    fn usage(&self) -> &str {
        "/pipes [subcommand]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "管道",
            serde_json::json!({"action": "pipes", "args": args, "session_id": ctx.session_id}),
        )
    }
}
