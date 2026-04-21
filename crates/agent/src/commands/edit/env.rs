//! /env 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct EnvCommand;
impl EnvCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for EnvCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for EnvCommand {
    fn name(&self) -> &str {
        "env"
    }
    fn description(&self) -> &str {
        "环境变量"
    }
    fn usage(&self) -> &str {
        "/env [key] [value]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "环境变量",
            serde_json::json!({"action": "env", "args": args, "session_id": ctx.session_id}),
        )
    }
}
