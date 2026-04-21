//! /model 命令 - 切换模型

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /model 命令
pub struct ModelCommand;

impl ModelCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModelCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ModelCommand {
    fn name(&self) -> &str {
        "model"
    }

    fn description(&self) -> &str {
        "切换 AI 模型"
    }

    fn usage(&self) -> &str {
        "/model [model-name]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        if let Some(model) = args.first() {
            let model_name = model.to_string();
            tracing::info!("Switching to model: {}", model_name);

            CommandResult::success_with_data(
                format!("已切换到模型: {}", model_name),
                serde_json::json!({
                    "action": "model_switch",
                    "model": model_name,
                    "session_id": ctx.session_id
                }),
            )
        } else {
            CommandResult::success_with_data(
                "当前模型信息",
                serde_json::json!({
                    "current_model": "claude-sonnet-4-20250514"
                }),
            )
        }
    }
}
