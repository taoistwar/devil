//! /release-notes 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ReleaseNotesCommand;
impl ReleaseNotesCommand { pub fn new() -> Self { Self } }
impl Default for ReleaseNotesCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for ReleaseNotesCommand {
    fn name(&self) -> &str { "release-notes" }
    fn description(&self) -> &str { "发布说明" }
    fn usage(&self) -> &str { "/release-notes" }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("发布说明", serde_json::json!({"action": "release-notes", "session_id": ctx.session_id}))
    }
}
