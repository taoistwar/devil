//! /plugin 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PluginCommand;
impl PluginCommand { pub fn new() -> Self { Self } }
impl Default for PluginCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for PluginCommand {
    fn name(&self) -> &str { "plugin" }
    fn description(&self) -> &str { "插件管理" }
    fn usage(&self) -> &str { "/plugin [subcommand]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("插件管理", serde_json::json!({"action": "plugin", "args": args, "session_id": ctx.session_id}))
    }
}
