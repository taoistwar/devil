//! /keybindings 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct KeybindingsCommand;
impl KeybindingsCommand { pub fn new() -> Self { Self } }
impl Default for KeybindingsCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for KeybindingsCommand {
    fn name(&self) -> &str { "keybindings" }
    fn description(&self) -> &str { "快捷键" }
    fn usage(&self) -> &str { "/keybindings" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("快捷键列表", serde_json::json!({"action": "keybindings", "session_id": ctx.session_id}))
    }
}
