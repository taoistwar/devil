//! /peers 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PeersCommand;
impl PeersCommand { pub fn new() -> Self { Self } }
impl Default for PeersCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for PeersCommand {
    fn name(&self) -> &str { "peers" }
    fn description(&self) -> &str { "对等连接" }
    fn usage(&self) -> &str { "/peers" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("对等连接", serde_json::json!({"action": "peers", "session_id": ctx.session_id}))
    }
}
