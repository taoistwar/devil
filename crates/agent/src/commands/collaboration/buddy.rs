//! /buddy 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct BuddyCommand;
impl BuddyCommand { pub fn new() -> Self { Self } }
impl Default for BuddyCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for BuddyCommand {
    fn name(&self) -> &str { "buddy" }
    fn description(&self) -> &str { "Buddy 模式" }
    fn usage(&self) -> &str { "/buddy" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("Buddy 模式", serde_json::json!({"action": "buddy", "session_id": ctx.session_id}))
    }
}
