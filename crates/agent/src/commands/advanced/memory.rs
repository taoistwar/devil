//! /memory 命令 - 记忆管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct MemoryCommand;

impl MemoryCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for MemoryCommand {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "记忆管理"
    }

    fn usage(&self) -> &str {
        "/memory [subcommand]"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "记忆管理",
            serde_json::json!({
                "action": "memory",
                "session_id": ctx.session_id,
                "entries": []
            }),
        )
    }
}
