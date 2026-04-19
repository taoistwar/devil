//! /color 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ColorCommand;
impl ColorCommand { pub fn new() -> Self { Self } }
impl Default for ColorCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for ColorCommand {
    fn name(&self) -> &str { "color" }
    fn description(&self) -> &str { "颜色配置" }
    fn usage(&self) -> &str { "/color [theme]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("颜色配置", serde_json::json!({"action": "color", "args": args, "session_id": ctx.session_id}))
    }
}
