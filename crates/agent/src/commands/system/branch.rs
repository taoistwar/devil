//! /branch 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct BranchCommand;
impl BranchCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for BranchCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for BranchCommand {
    fn name(&self) -> &str {
        "branch"
    }
    fn description(&self) -> &str {
        "分支管理"
    }
    fn usage(&self) -> &str {
        "/branch [name]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "分支",
            serde_json::json!({"action": "branch", "args": args, "session_id": ctx.session_id}),
        )
    }
}
