//! /pr_comments 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PrCommentsCommand;
impl PrCommentsCommand { pub fn new() -> Self { Self } }
impl Default for PrCommentsCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for PrCommentsCommand {
    fn name(&self) -> &str { "pr_comments" }
    fn description(&self) -> &str { "PR 评论" }
    fn usage(&self) -> &str { "/pr_comments [pr-url]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("PR 评论", serde_json::json!({"action": "pr_comments", "args": args, "session_id": ctx.session_id}))
    }
}
