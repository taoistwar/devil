//! /extra-usage 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ExtraUsageCommand;
impl ExtraUsageCommand { pub fn new() -> Self { Self } }
impl Default for ExtraUsageCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for ExtraUsageCommand {
    fn name(&self) -> &str { "extra-usage" }
    fn description(&self) -> &str { "额外使用量" }
    fn usage(&self) -> &str { "/extra-usage" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("额外使用量", serde_json::json!({"action": "extra-usage", "session_id": ctx.session_id}))
    }
}
