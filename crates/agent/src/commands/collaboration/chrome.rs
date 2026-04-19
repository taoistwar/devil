//! /chrome 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ChromeCommand;
impl ChromeCommand { pub fn new() -> Self { Self } }
impl Default for ChromeCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for ChromeCommand {
    fn name(&self) -> &str { "chrome" }
    fn description(&self) -> &str { "Chrome 集成" }
    fn usage(&self) -> &str { "/chrome" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("Chrome 集成", serde_json::json!({"action": "chrome", "session_id": ctx.session_id}))
    }
}
