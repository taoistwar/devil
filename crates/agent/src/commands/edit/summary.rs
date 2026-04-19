//! /summary 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct SummaryCommand;
impl SummaryCommand { pub fn new() -> Self { Self } }
impl Default for SummaryCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for SummaryCommand {
    fn name(&self) -> &str { "summary" }
    fn description(&self) -> &str { "摘要生成" }
    fn usage(&self) -> &str { "/summary" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("摘要", serde_json::json!({"action": "summary", "session_id": ctx.session_id}))
    }
}
