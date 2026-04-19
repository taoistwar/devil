//! /attach 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct AttachCommand;
impl AttachCommand { pub fn new() -> Self { Self } }
impl Default for AttachCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for AttachCommand {
    fn name(&self) -> &str { "attach" }
    fn description(&self) -> &str { "附加内容" }
    fn usage(&self) -> &str { "/attach [file]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("内容已附加", serde_json::json!({"action": "attach", "args": args, "session_id": ctx.session_id}))
    }
}
