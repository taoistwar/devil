//! 上下文管理模块
//! 
//! 实现 Claude Code 的四级渐进式上下文压缩策略：
//! 
//! # 四级压缩策略（从低成本到高成本）
//! 
//! 1. **Snip（裁剪）** - 标记清除旧的工具结果内容，无 LLM 调用
//! 2. **MicroCompact（微压缩）** - 基于时间触发的缓存过期清理
//! 3. **Collapse（折叠）** - 主动重构上下文，部分 LLM 调用
//! 4. **AutoCompact（自动压缩）** - 完整对话摘要，LLM 调用
//! 
//! # 有效窗口公式
//! 
//! ```
//! 有效窗口 = 模型窗口 - 预留输出令牌
//! 预留令牌 = min(模型最大输出令牌，20,000)
//! ```
//! 
//! # 断路器设计
//! 
//! 连续 3 次压缩失败后停止尝试，避免 API 调用的雪崩效应
//! 
//! # 令牌预算追踪
//! 
//! - 50,000 总预算
//! - 5,000 每文件
//! - 25,000 技能独立预算

use anyhow::Result;
use crate::message::Message;

pub mod context_window;
pub mod compression;
pub mod circuit_breaker;
pub mod token_budget;

pub use context_window::*;
pub use compression::*;
pub use circuit_breaker::*;
pub use token_budget::*;

/// 上下文管线处理结果
#[derive(Debug)]
pub enum ContextPipelineResult {
    Success {
        messages: Vec<Message>,
        system_prompt: String,
        token_count: usize,
    },
    TokenLimitExceeded {
        current_tokens: usize,
        max_tokens: usize,
    },
}

/// 上下文管理器
#[derive(Clone)]
pub struct ContextManager {
    max_context_tokens: usize,
}

impl ContextManager {
    pub fn with_defaults() -> Self {
        Self {
            max_context_tokens: 100000,
        }
    }

    pub async fn process_full_pipeline(
        &self,
        messages: Vec<Message>,
        system_prompt: &str,
        max_tokens: usize,
    ) -> Result<ContextPipelineResult> {
        let token_count = messages.len() * 100;
        if token_count > max_tokens {
            return Ok(ContextPipelineResult::TokenLimitExceeded {
                current_tokens: token_count,
                max_tokens,
            });
        }
        Ok(ContextPipelineResult::Success {
            messages,
            system_prompt: system_prompt.to_string(),
            token_count,
        })
    }
}
