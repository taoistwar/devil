//! /good-claude 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct GoodClaudeCommand;
impl GoodClaudeCommand { pub fn new() -> Self { Self } }
impl Default for GoodClaudeCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for GoodClaudeCommand {
    fn name(&self) -> &str { "good-claude" }
    fn description(&self) -> &str { "反馈好的体验" }
    fn usage(&self) -> &str { "/good-claude [comment]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("感谢反馈!", serde_json::json!({"action": "good-claude", "args": args, "session_id": ctx.session_id}))
    }
}
