//! /agents 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct AgentsCommand;
impl AgentsCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for AgentsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for AgentsCommand {
    fn name(&self) -> &str {
        "agents"
    }
    fn description(&self) -> &str {
        "多代理管理"
    }
    fn usage(&self) -> &str {
        "/agents [subcommand]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "代理管理",
            serde_json::json!({"action": "agents", "args": args, "session_id": ctx.session_id}),
        )
    }
}
