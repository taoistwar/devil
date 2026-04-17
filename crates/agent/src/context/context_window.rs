//! 上下文窗口管理模块
//! 
//! 实现有效上下文窗口计算和令牌使用追踪

use serde::{Deserialize, Serialize};

/// 默认最大输出令牌预留（用于压缩摘要）
pub const MAX_OUTPUT_TOKENS_FOR_SUMMARY: u32 = 20_000;

/// 自动压缩缓冲区令牌
pub const AUTOCOMPACT_BUFFER_TOKENS: u32 = 13_000;

/// 警告阈值缓冲区令牌
pub const WARNING_THRESHOLD_BUFFER_TOKENS: u32 = 20_000;

/// 错误阈值缓冲区令牌
pub const ERROR_THRESHOLD_BUFFER_TOKENS: u32 = 20_000;

/// 手动压缩缓冲区令牌
pub const MANUAL_COMPACT_BUFFER_TOKENS: u32 = 3_000;

/// 断路器：连续压缩失败最大次数
pub const MAX_CONSECUTIVE_AUTOCOMPACT_FAILURES: u32 = 3;

/// 上下文窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindowConfig {
    /// 模型上下文窗口大小（令牌数）
    pub model_window: u32,
    /// 模型最大输出令牌数
    pub max_output_tokens: u32,
    /// 环境变量覆盖（可选）
    pub env_override_window: Option<u32>,
}

impl Default for ContextWindowConfig {
    fn default() -> Self {
        Self {
            // Claude 3.5 Sonnet 默认 200K 上下文
            model_window: 200_000,
            max_output_tokens: 16_384,
            env_override_window: None,
        }
    }
}

/// 有效上下文窗口计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveWindowSize {
    /// 模型上下文窗口
    pub model_window: u32,
    /// 预留输出令牌
    pub reserved_tokens: u32,
    /// 有效窗口大小
    pub effective_window: u32,
}

impl EffectiveWindowSize {
    /// 计算有效上下文窗口
    /// 
    /// 公式：有效窗口 = 模型窗口 - 预留输出令牌
    /// 预留令牌 = min(模型最大输出令牌，20,000)
    pub fn calculate(config: &ContextWindowConfig) -> Self {
        let reserved_tokens = config.max_output_tokens.min(MAX_OUTPUT_TOKENS_FOR_SUMMARY);
        
        let model_window = config.env_override_window
            .map(|env_window| env_window.min(config.model_window))
            .unwrap_or(config.model_window);
        
        let effective_window = model_window.saturating_sub(reserved_tokens);
        
        Self {
            model_window,
            reserved_tokens,
            effective_window,
        }
    }
}

/// 令牌使用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageState {
    /// 当前令牌使用量
    pub current_usage: u32,
    /// 有效窗口大小
    pub effective_window: u32,
    /// 剩余百分比（0-100）
    pub percent_left: u32,
    /// 是否超过警告阈值
    pub is_above_warning_threshold: bool,
    /// 是否超过错误阈值
    pub is_above_error_threshold: bool,
    /// 是否超过自动压缩阈值
    pub is_above_auto_compact_threshold: bool,
    /// 是否达到阻塞限制
    pub is_at_blocking_limit: bool,
}

impl TokenUsageState {
    /// 计算令牌警告状态
    /// 
    /// # 参数
    /// 
    /// * `current_usage` - 当前令牌使用量
    /// * `effective_window` - 有效窗口大小
    /// * `auto_compact_enabled` - 是否启用自动压缩
    pub fn calculate(
        current_usage: u32,
        effective_window: u32,
        auto_compact_enabled: bool,
    ) -> Self {
        // 自动压缩阈值
        let auto_compact_threshold = if auto_compact_enabled {
            effective_window.saturating_sub(AUTOCOMPACT_BUFFER_TOKENS)
        } else {
            effective_window
        };

        // 警告和错误阈值
        let warning_threshold = effective_window.saturating_sub(WARNING_THRESHOLD_BUFFER_TOKENS);
        let error_threshold = effective_window.saturating_sub(ERROR_THRESHOLD_BUFFER_TOKENS);
        
        // 阻塞限制
        let blocking_limit = effective_window.saturating_sub(MANUAL_COMPACT_BUFFER_TOKENS);

        // 剩余百分比
        let percent_left = if effective_window > 0 {
            ((effective_window.saturating_sub(current_usage) as f32) 
                / effective_window as f32 * 100.0)
                .round() as u32
        } else {
            0
        };

        Self {
            current_usage,
            effective_window,
            percent_left,
            is_above_warning_threshold: current_usage >= warning_threshold,
            is_above_error_threshold: current_usage >= error_threshold,
            is_above_auto_compact_threshold: auto_compact_enabled 
                && current_usage >= auto_compact_threshold,
            is_at_blocking_limit: current_usage >= blocking_limit,
        }
    }

    /// 获取自动压缩阈值
    pub fn get_auto_compact_threshold(effective_window: u32) -> u32 {
        effective_window.saturating_sub(AUTOCOMPACT_BUFFER_TOKENS)
    }

    /// 判断是否应该触发自动压缩
    pub fn should_trigger_auto_compact(&self) -> bool {
        self.is_above_auto_compact_threshold && !self.is_at_blocking_limit
    }

    /// 判断是否应该阻止新请求
    pub fn should_block_new_request(&self) -> bool {
        self.is_at_blocking_limit
    }
}

/// 压缩触发类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactionTrigger {
    /// 用户手动触发
    Manual,
    /// 自动触发（令牌阈值）
    Auto,
    /// 时间触发（缓存过期）
    TimeBased,
    /// 空间压力触发
    SpacePressure,
}

/// 压缩级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Level 1: Snip（裁剪）- 无 LLM 调用
    Snip,
    /// Level 2: MicroCompact（微压缩）- 无 LLM 调用
    MicroCompact,
    /// Level 3: Collapse（折叠）- 部分 LLM 调用
    Collapse,
    /// Level 4: AutoCompact（自动压缩）- 完整 LLM 调用
    AutoCompact,
}

impl CompressionLevel {
    /// 判断是否需要 LLM 调用
    pub fn requires_llm(&self) -> bool {
        matches!(self, Self::Collapse | Self::AutoCompact)
    }

    /// 获取压缩成本等级（0-3）
    pub fn cost_level(&self) -> u8 {
        match self {
            Self::Snip => 0,
            Self::MicroCompact => 1,
            Self::Collapse => 2,
            Self::AutoCompact => 3,
        }
    }
}

/// 上下文窗口升级检查
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindowUpgrade {
    /// 是否可用升级
    pub available: bool,
    /// 升级后的窗口大小
    pub upgraded_window: Option<u32>,
    /// 升级条件描述
    pub requirements: String,
}

impl ContextWindowUpgrade {
    /// 检查是否可用升级
    pub fn check_available(current_window: u32, model: &str) -> Self {
        // 简化实现：某些模型支持更大的上下文窗口
        let available = model.contains("sonnet") || model.contains("opus");
        let upgraded_window = if available {
            Some(current_window.max(200_000))
        } else {
            None
        };

        Self {
            available,
            upgraded_window,
            requirements: if available {
                "使用 Claude 3.5 Sonnet 或 Opus".to_string()
            } else {
                "当前模型不支持大上下文".to_string()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_window_calculation() {
        let config = ContextWindowConfig::default();
        let result = EffectiveWindowSize::calculate(&config);

        // 200,000 - 16,384 = 183,616
        assert_eq!(result.model_window, 200_000);
        assert_eq!(result.reserved_tokens, 16_384);
        assert_eq!(result.effective_window, 183_616);
    }

    #[test]
    fn test_token_usage_state_warning() {
        let effective_window = 183_616;
        let current_usage = effective_window - WARNING_THRESHOLD_BUFFER_TOKENS + 1;

        let state = TokenUsageState::calculate(current_usage, effective_window, true);

        assert!(state.is_above_warning_threshold);
        assert!(!state.is_above_error_threshold);
        assert_eq!(state.percent_left, 11); // 约 11% 剩余
    }

    #[test]
    fn test_token_usage_state_auto_compact() {
        let effective_window = 183_616;
        let current_usage = effective_window - AUTOCOMPACT_BUFFER_TOKENS + 1;

        let state = TokenUsageState::calculate(current_usage, effective_window, true);

        assert!(state.is_above_auto_compact_threshold);
        assert!(!state.is_at_blocking_limit);
        assert!(state.should_trigger_auto_compact());
    }

    #[test]
    fn test_token_usage_state_blocking() {
        let effective_window = 183_616;
        let current_usage = effective_window - MANUAL_COMPACT_BUFFER_TOKENS + 1;

        let state = TokenUsageState::calculate(current_usage, effective_window, true);

        assert!(state.is_at_blocking_limit);
        assert!(state.should_block_new_request());
    }

    #[test]
    fn test_compression_level_cost() {
        assert_eq!(CompressionLevel::Snip.cost_level(), 0);
        assert_eq!(CompressionLevel::MicroCompact.cost_level(), 1);
        assert_eq!(CompressionLevel::Collapse.cost_level(), 2);
        assert_eq!(CompressionLevel::AutoCompact.cost_level(), 3);
    }

    #[test]
    fn test_compression_requires_llm() {
        assert!(!CompressionLevel::Snip.requires_llm());
        assert!(!CompressionLevel::MicroCompact.requires_llm());
        assert!(CompressionLevel::Collapse.requires_llm());
        assert!(CompressionLevel::AutoCompact.requires_llm());
    }

    #[test]
    fn test_context_window_upgrade() {
        let upgrade = ContextWindowUpgrade::check_available(100_000, "claude-3-5-sonnet");
        assert!(upgrade.available);
        assert_eq!(upgrade.upgraded_window, Some(200_000));

        let upgrade = ContextWindowUpgrade::check_available(100_000, "claude-3-haiku");
        assert!(!upgrade.available);
        assert_eq!(upgrade.upgraded_window, None);
    }
}
