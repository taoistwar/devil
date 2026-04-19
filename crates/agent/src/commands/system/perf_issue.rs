//! /perf-issue 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PerfIssueCommand;
impl PerfIssueCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PerfIssueCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for PerfIssueCommand {
    fn name(&self) -> &str {
        "perf-issue"
    }
    fn description(&self) -> &str {
        "性能问题"
    }
    fn usage(&self) -> &str {
        "/perf-issue"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "性能问题",
            serde_json::json!({"action": "perf-issue", "session_id": ctx.session_id}),
        )
    }
}
