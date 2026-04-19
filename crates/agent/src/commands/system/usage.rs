//! /usage 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct UsageCommand;
impl UsageCommand { pub fn new() -> Self { Self } }
impl Default for UsageCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for UsageCommand {
    fn name(&self) -> &str { "usage" }
    fn description(&self) -> &str { "使用统计" }
    fn usage(&self) -> &str { "/usage" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("使用统计", serde_json::json!({"action": "usage", "session_id": ctx.session_id}))
    }
}
