//! /workflows 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct WorkflowsCommand;
impl WorkflowsCommand { pub fn new() -> Self { Self } }
impl Default for WorkflowsCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for WorkflowsCommand {
    fn name(&self) -> &str { "workflows" }
    fn description(&self) -> &str { "工作流管理" }
    fn usage(&self) -> &str { "/workflows [subcommand]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("工作流", serde_json::json!({"action": "workflows", "args": args, "session_id": ctx.session_id}))
    }
}
