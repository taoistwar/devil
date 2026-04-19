//! /feedback 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct FeedbackCommand;
impl FeedbackCommand { pub fn new() -> Self { Self } }
impl Default for FeedbackCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for FeedbackCommand {
    fn name(&self) -> &str { "feedback" }
    fn description(&self) -> &str { "反馈" }
    fn usage(&self) -> &str { "/feedback [text]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("感谢反馈", serde_json::json!({"action": "feedback", "args": args, "session_id": ctx.session_id}))
    }
}
