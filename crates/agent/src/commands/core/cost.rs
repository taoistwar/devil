//! /cost 命令 - 查看费用

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /cost 命令
pub struct CostCommand;

impl CostCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CostCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for CostCommand {
    fn name(&self) -> &str {
        "cost"
    }

    fn description(&self) -> &str {
        "查看 API 使用费用"
    }

    fn usage(&self) -> &str {
        "/cost"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Cost report requested for session: {}", ctx.session_id);

        let report = r#"
API 使用费用报告
================

输入 tokens: 0
输出 tokens: 0
预估费用: $0.00

详细统计:
  - claude-sonnet-4: $0.00
  - claude-opus: $0.00

查看完整报告: /usage
"#;

        CommandResult::success_with_data(
            report.trim(),
            serde_json::json!({
                "action": "cost",
                "session_id": ctx.session_id,
                "input_tokens": 0,
                "output_tokens": 0,
                "estimated_cost": "0.00"
            }),
        )
    }
}
