//! /bridge 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct BridgeCommand;
impl BridgeCommand { pub fn new() -> Self { Self } }
impl Default for BridgeCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for BridgeCommand {
    fn name(&self) -> &str { "bridge" }
    fn description(&self) -> &str { "桥接模式" }
    fn usage(&self) -> &str { "/bridge" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("桥接模式", serde_json::json!({"action": "bridge", "session_id": ctx.session_id}))
    }
}
