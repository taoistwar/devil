//! /files 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct FilesCommand;
impl FilesCommand { pub fn new() -> Self { Self } }
impl Default for FilesCommand { fn default() -> Self { Self::new() } }

#[async_trait]
impl SlashCommand for FilesCommand {
    fn name(&self) -> &str { "files" }
    fn description(&self) -> &str { "文件管理" }
    fn usage(&self) -> &str { "/files [subcommand]" }
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        CommandResult::success_with_data("文件列表", serde_json::json!({"action": "files", "args": args, "session_id": ctx.session_id}))
    }
}
