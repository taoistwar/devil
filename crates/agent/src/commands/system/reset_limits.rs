//! /reset-limits 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ResetLimitsCommand;
impl ResetLimitsCommand { pub fn new() -> Self { Self } }
impl Default for ResetLimitsCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for ResetLimitsCommand {
    fn name(&self) -> &str { "reset-limits" }
    fn description(&self) -> &str { "重置限制" }
    fn usage(&self) -> &str { "/reset-limits" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("限制已重置", serde_json::json!({"action": "reset-limits", "session_id": ctx.session_id}))
    }
}
