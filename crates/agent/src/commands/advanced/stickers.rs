//! /stickers 命令 - 贴纸（彩蛋）

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct StickersCommand;

impl StickersCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StickersCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for StickersCommand {
    fn name(&self) -> &str {
        "stickers"
    }

    fn description(&self) -> &str {
        "贴纸（彩蛋）"
    }

    fn usage(&self) -> &str {
        "/stickers [sticker-name]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let sticker = args.first().map(|s| s.to_string()).unwrap_or_else(|| "thumbs_up".to_string());

        CommandResult::success_with_data(
            format!("发送贴纸: {}", sticker),
            serde_json::json!({
                "action": "stickers",
                "sticker": sticker,
                "session_id": ctx.session_id
            }),
        )
    }
}
