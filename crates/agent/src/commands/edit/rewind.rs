//! /rewind 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct RewindCommand;
impl RewindCommand { pub fn new() -> Self { Self } }
impl Default for RewindCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for RewindCommand {
    fn name(&self) -> &str { "rewind" }
    fn description(&self) -> &str { "回退会话" }
    fn usage(&self) -> &str { "/rewind [n]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let n = args.first().map(|s| s.to_string()).unwrap_or_else(|| "1".to_string());
        CommandResult::success_with_data(format!("回退 {} 步", n), serde_json::json!({"action": "rewind", "steps": n, "session_id": ctx.session_id}))
    }
}
