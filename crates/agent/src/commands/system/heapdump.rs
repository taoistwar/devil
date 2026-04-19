//! /heapdump 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct HeapdumpCommand;
impl HeapdumpCommand { pub fn new() -> Self { Self } }
impl Default for HeapdumpCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for HeapdumpCommand {
    fn name(&self) -> &str { "heapdump" }
    fn description(&self) -> &str { "堆转储" }
    fn usage(&self) -> &str { "/heapdump" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("堆转储", serde_json::json!({"action": "heapdump", "session_id": ctx.session_id}))
    }
}
