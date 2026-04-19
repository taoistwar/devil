//! /issue 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct IssueCommand;
impl IssueCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for IssueCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for IssueCommand {
    fn name(&self) -> &str {
        "issue"
    }
    fn description(&self) -> &str {
        "问题管理"
    }
    fn usage(&self) -> &str {
        "/issue [subcommand]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "问题",
            serde_json::json!({"action": "issue", "args": args, "session_id": ctx.session_id}),
        )
    }
}
