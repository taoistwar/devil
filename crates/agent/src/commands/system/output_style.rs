//! /output-style 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct OutputStyleCommand;
impl OutputStyleCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for OutputStyleCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for OutputStyleCommand {
    fn name(&self) -> &str {
        "output-style"
    }
    fn description(&self) -> &str {
        "输出样式"
    }
    fn usage(&self) -> &str {
        "/output-style [style]"
    }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "输出样式",
            serde_json::json!({"action": "output-style", "args": args, "session_id": ctx.session_id}),
        )
    }
}
