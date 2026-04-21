//! Agent 配置模块
//!
//! 定义 Agent 的配置参数，包括：
//! - 模型设置
//! - Token 预算
//! - 最大循环次数
//! - 压缩策略配置

use serde::{Deserialize, Serialize};

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent 名称
    pub name: String,
    /// 模型名称
    pub model: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 最大连续对话轮数
    pub max_turns: usize,
    /// 上下文 Token 预算上限
    pub max_context_tokens: usize,
    /// 自动压缩触发阈值（Token 数）
    pub auto_compact_threshold: usize,
    /// 输出 Token 最大恢复次数
    pub max_output_token_recovery: usize,
    /// 是否启用流式工具执行
    pub enable_streaming_tools: bool,
    /// 是否启用历史裁剪
    pub enable_history_snip: bool,
    /// 是否启用 MicroCompact
    pub enable_micro_compact: bool,
    /// 是否启用 Context Collapse
    pub enable_context_collapse: bool,
    /// 是否启用 AutoCompact
    pub enable_auto_compact: bool,
}

/// Agent 配置构建器
#[derive(Default)]
pub struct AgentConfigBuilder {
    name: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    max_turns: Option<usize>,
    max_context_tokens: Option<usize>,
    auto_compact_threshold: Option<usize>,
    max_output_token_recovery: Option<usize>,
    enable_streaming_tools: Option<bool>,
    enable_history_snip: Option<bool>,
    enable_micro_compact: Option<bool>,
    enable_context_collapse: Option<bool>,
    enable_auto_compact: Option<bool>,
}

impl AgentConfigBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 Agent 名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置模型名称
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// 设置系统提示词
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// 设置最大连续对话轮数
    pub fn max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = Some(max_turns);
        self
    }

    /// 设置上下文 Token 预算上限
    pub fn max_context_tokens(mut self, tokens: usize) -> Self {
        self.max_context_tokens = Some(tokens);
        self
    }

    /// 设置自动压缩触发阈值
    pub fn auto_compact_threshold(mut self, tokens: usize) -> Self {
        self.auto_compact_threshold = Some(tokens);
        self
    }

    /// 设置输出 Token 最大恢复次数
    pub fn max_output_token_recovery(mut self, count: usize) -> Self {
        self.max_output_token_recovery = Some(count);
        self
    }

    /// 设置是否启用流式工具执行
    pub fn enable_streaming_tools(mut self, enable: bool) -> Self {
        self.enable_streaming_tools = Some(enable);
        self
    }

    /// 设置是否启用历史裁剪
    pub fn enable_history_snip(mut self, enable: bool) -> Self {
        self.enable_history_snip = Some(enable);
        self
    }

    /// 设置是否启用 MicroCompact
    pub fn enable_micro_compact(mut self, enable: bool) -> Self {
        self.enable_micro_compact = Some(enable);
        self
    }

    /// 设置是否启用 Context Collapse
    pub fn enable_context_collapse(mut self, enable: bool) -> Self {
        self.enable_context_collapse = Some(enable);
        self
    }

    /// 设置是否启用 AutoCompact
    pub fn enable_auto_compact(mut self, enable: bool) -> Self {
        self.enable_auto_compact = Some(enable);
        self
    }

    /// 构建配置
    pub fn build(self) -> AgentConfig {
        AgentConfig {
            name: self.name.unwrap_or_else(|| "agent".to_string()),
            model: self
                .model
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
            system_prompt: self.system_prompt.unwrap_or_default(),
            max_turns: self.max_turns.unwrap_or(50),
            max_context_tokens: self.max_context_tokens.unwrap_or(200000),
            auto_compact_threshold: self.auto_compact_threshold.unwrap_or(100000),
            max_output_token_recovery: self.max_output_token_recovery.unwrap_or(3),
            enable_streaming_tools: self.enable_streaming_tools.unwrap_or(true),
            enable_history_snip: self.enable_history_snip.unwrap_or(true),
            enable_micro_compact: self.enable_micro_compact.unwrap_or(true),
            enable_context_collapse: self.enable_context_collapse.unwrap_or(true),
            enable_auto_compact: self.enable_auto_compact.unwrap_or(true),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "agent".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            system_prompt: String::new(),
            max_turns: 50,
            max_context_tokens: 200000,
            auto_compact_threshold: 100000,
            max_output_token_recovery: 3,
            enable_streaming_tools: true,
            enable_history_snip: true,
            enable_micro_compact: true,
            enable_context_collapse: true,
            enable_auto_compact: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = AgentConfigBuilder::new()
            .name("test-agent")
            .model("claude-3-opus")
            .max_turns(100)
            .build();

        assert_eq!(config.name, "test-agent");
        assert_eq!(config.model, "claude-3-opus");
        assert_eq!(config.max_turns, 100);
    }

    #[test]
    fn test_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_turns, 50);
        assert!(config.enable_streaming_tools);
    }
}
