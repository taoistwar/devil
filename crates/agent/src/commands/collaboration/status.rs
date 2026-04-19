//! /status 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct StatusCommand;
impl StatusCommand { pub fn new() -> Self { Self } }
impl Default for StatusCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for StatusCommand {
    fn name(&self) -> &str { "status" }
    fn description(&self) -> &str { "状态查看" }
    fn usage(&self) -> &str { "/status" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("状态", serde_json::json!({"action": "status", "session_id": ctx.session_id}))
    }
}
