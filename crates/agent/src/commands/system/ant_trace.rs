//! /ant-trace 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct AntTraceCommand;
impl AntTraceCommand { pub fn new() -> Self { Self } }
impl Default for AntTraceCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for AntTraceCommand {
    fn name(&self) -> &str { "ant-trace" }
    fn description(&self) -> &str { "Ant 追踪" }
    fn usage(&self) -> &str { "/ant-trace" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("Ant 追踪", serde_json::json!({"action": "ant-trace", "session_id": ctx.session_id}))
    }
}
