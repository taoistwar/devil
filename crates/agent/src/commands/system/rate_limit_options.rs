//! /rate-limit-options 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct RateLimitOptionsCommand;
impl RateLimitOptionsCommand { pub fn new() -> Self { Self } }
impl Default for RateLimitOptionsCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for RateLimitOptionsCommand {
    fn name(&self) -> &str { "rate-limit-options" }
    fn description(&self) -> &str { "速率限制选项" }
    fn usage(&self) -> &str { "/rate-limit-options" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("速率限制选项", serde_json::json!({"action": "rate-limit-options", "session_id": ctx.session_id}))
    }
}
