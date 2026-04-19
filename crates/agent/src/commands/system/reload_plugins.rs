//! /reload-plugins 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ReloadPluginsCommand;
impl ReloadPluginsCommand { pub fn new() -> Self { Self } }
impl Default for ReloadPluginsCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for ReloadPluginsCommand {
    fn name(&self) -> &str { "reload-plugins" }
    fn description(&self) -> &str { "重载插件" }
    fn usage(&self) -> &str { "/reload-plugins" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("插件已重载", serde_json::json!({"action": "reload-plugins", "session_id": ctx.session_id}))
    }
}
