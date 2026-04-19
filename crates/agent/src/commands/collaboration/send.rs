//! /send 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct SendCommand;
impl SendCommand { pub fn new() -> Self { Self } }
impl Default for SendCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for SendCommand {
    fn name(&self) -> &str { "send" }
    fn description(&self) -> &str { "发送消息" }
    fn usage(&self) -> &str { "/send [message]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("消息已发送", serde_json::json!({"action": "send", "args": args, "session_id": ctx.session_id}))
    }
}
