//! /add-dir 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct AddDirCommand;
impl AddDirCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for AddDirCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for AddDirCommand {
    fn name(&self) -> &str {
        "add-dir"
    }
    fn description(&self) -> &str {
        "添加目录"
    }
    fn usage(&self) -> &str {
        "/add-dir <path>"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "目录已添加",
            serde_json::json!({"action": "add-dir", "args": args, "session_id": ctx.session_id}),
        )
    }
}
