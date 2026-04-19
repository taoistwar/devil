//! /src 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct SrcCommand;
impl SrcCommand { pub fn new() -> Self { Self } }
impl Default for SrcCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for SrcCommand {
    fn name(&self) -> &str { "src" }
    fn description(&self) -> &str { "源代码相关" }
    fn usage(&self) -> &str { "/src [subcommand]" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("源代码管理", serde_json::json!({"action": "src", "session_id": ctx.session_id}))
    }
}
