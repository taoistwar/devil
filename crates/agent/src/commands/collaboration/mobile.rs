//! /mobile 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct MobileCommand;
impl MobileCommand { pub fn new() -> Self { Self } }
impl Default for MobileCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for MobileCommand {
    fn name(&self) -> &str { "mobile" }
    fn description(&self) -> &str { "移动端" }
    fn usage(&self) -> &str { "/mobile" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("移动端", serde_json::json!({"action": "mobile", "session_id": ctx.session_id}))
    }
}
