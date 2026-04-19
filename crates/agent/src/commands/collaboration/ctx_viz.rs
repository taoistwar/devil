//! /ctx_viz 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct CtxVizCommand;
impl CtxVizCommand { pub fn new() -> Self { Self } }
impl Default for CtxVizCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for CtxVizCommand {
    fn name(&self) -> &str { "ctx_viz" }
    fn description(&self) -> &str { "上下文可视化" }
    fn usage(&self) -> &str { "/ctx_viz" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("上下文可视化", serde_json::json!({"action": "ctx_viz", "session_id": ctx.session_id}))
    }
}
