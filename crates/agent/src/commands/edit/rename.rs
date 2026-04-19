//! /rename 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct RenameCommand;
impl RenameCommand { pub fn new() -> Self { Self } }
impl Default for RenameCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for RenameCommand {
    fn name(&self) -> &str { "rename" }
    fn description(&self) -> &str { "重命名会话" }
    fn usage(&self) -> &str { "/rename [name]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("会话已重命名", serde_json::json!({"action": "rename", "args": args, "session_id": ctx.session_id}))
    }
}
