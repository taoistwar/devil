//! 状态转换模块
//!
//! 定义对话循环的核心状态机：
//! - State: 可变循环状态
//! - Continue: 继续循环的决策
//! - Terminal: 终止信号

use crate::message::Message;
use crate::tools::ToolContext;
use serde::{Deserialize, Serialize};

/// 对话循环的完整可变状态
///
/// 每次 continue 都创建新实例，确保状态不可变且转换可追踪
#[derive(Debug, Clone)]
pub struct State {
    /// 消息历史列表
    pub messages: Vec<Message>,
    /// 工具执行上下文
    pub tool_context: ToolContext,
    /// 自动压缩追踪状态
    pub auto_compact_state: AutoCompactState,
    /// 输出 token 恢复计数器
    pub output_token_recovery_count: usize,
    /// 是否已尝试响应式压缩
    pub has_tried_reactive_compact: bool,
    /// 输出 token 覆盖限制
    pub output_token_override: Option<usize>,
    /// 待处理的工具摘要
    pub pending_tool_summary: Option<String>,
    /// Stop hook 是否激活
    pub stop_hook_active: bool,
    /// 当前轮次计数
    pub turn_count: usize,
    /// 上一次继续循环的原因
    pub transition: TransitionState,
}

/// 自动压缩追踪状态
#[derive(Debug, Clone, Default)]
pub struct AutoCompactState {
    /// 是否已执行自动压缩
    pub has_compacted: bool,
    /// 压缩后的 token 数
    pub compacted_token_count: Option<usize>,
    /// 压缩尝试次数
    pub attempt_count: usize,
}

/// 上一次继续循环的原因
#[derive(Debug, Clone, Default)]
pub struct TransitionState {
    /// 继续原因
    pub reason: Option<ContinueReason>,
    /// 附加信息
    pub metadata: serde_json::Value,
}

impl State {
    /// 创建初始状态
    pub fn initial(messages: Vec<Message>) -> Self {
        Self {
            messages,
            tool_context: ToolContext::default(),
            auto_compact_state: AutoCompactState::default(),
            output_token_recovery_count: 0,
            has_tried_reactive_compact: false,
            output_token_override: None,
            pending_tool_summary: None,
            stop_hook_active: false,
            turn_count: 0,
            transition: TransitionState::default(),
        }
    }

    /// 创建下一个状态（用于 continue）
    pub fn next(&self, transition: ContinueReason, messages: Vec<Message>) -> Self {
        Self {
            messages,
            tool_context: self.tool_context.clone(),
            auto_compact_state: self.auto_compact_state.clone(),
            output_token_recovery_count: self.output_token_recovery_count,
            has_tried_reactive_compact: self.has_tried_reactive_compact,
            output_token_override: self.output_token_override,
            pending_tool_summary: self.pending_tool_summary.clone(),
            stop_hook_active: self.stop_hook_active,
            turn_count: self.turn_count + 1,
            transition: TransitionState {
                reason: Some(transition),
                metadata: serde_json::Value::Null,
            },
        }
    }
}

/// 终止原因枚举
///
/// 细粒度划分终止原因，便于调试和可观测性
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalReason {
    /// 正常完成
    Completed,
    /// 流式输出期间被用户中断
    AbortedStreaming,
    /// 工具执行期间被中断
    AbortedTools,
    /// 达到最大循环次数
    MaxTurns,
    /// Token 数超过硬性限制
    BlockingLimit,
    /// 上下文过长且恢复失败
    PromptTooLong,
    /// API 调用异常
    ModelError,
    /// Stop hook 阻止继续
    StopHookPrevented,
    /// 工具 hook 阻止继续
    HookStopped,
    /// 图片尺寸/格式错误
    ImageError,
    /// 输出 token 恢复耗尽
    OutputTokenExhausted,
    /// 工具执行失败
    ToolExecutionFailed,
}

/// 终止状态
///
/// 标记对话的终结，携带终止原因和附加信息
#[derive(Debug, Clone)]
pub struct Terminal {
    /// 终止原因
    pub reason: TerminalReason,
    /// 人类可读的消息
    pub message: Option<String>,
    /// 附加元数据
    pub metadata: serde_json::Value,
}

impl Terminal {
    /// 创建终止状态
    pub fn new(reason: TerminalReason) -> Self {
        Self {
            reason,
            message: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// 带消息的终止状态
    pub fn with_message(reason: TerminalReason, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: Some(message.into()),
            metadata: serde_json::Value::Null,
        }
    }

    /// 带元数据的终止状态
    pub fn with_metadata(reason: TerminalReason, metadata: serde_json::Value) -> Self {
        Self {
            reason,
            message: None,
            metadata,
        }
    }
}

/// 继续原因枚举
///
/// 每条 continue 路径都有明确的原因，用于状态转换追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinueReason {
    /// 正常的工具调用后继续（下一轮）
    NextTurn,
    /// 模型输出被截断，注入恢复消息后继续
    MaxOutputTokensRecovery,
    /// 首次截断时尝试提升输出 token 限制
    MaxOutputTokensEscalate,
    /// 上下文过长，通过响应式压缩恢复
    ReactiveCompactRetry,
    /// 上下文折叠的溢出恢复
    CollapseDrainRetry,
    /// Stop hook 返回阻塞错误，注入错误后继续
    StopHookBlocking,
    /// Token 预算管理触发的继续
    TokenBudgetContinuation,
    /// 工具执行失败后重试
    ToolRetry,
    /// 附件注入后继续
    AttachmentInjection,
}

/// 继续状态
///
/// 标记继续循环的决策，携带原因和可选的附加信息
#[derive(Debug, Clone)]
pub struct Continue {
    /// 继续原因
    pub reason: ContinueReason,
    /// 附加信息
    pub metadata: serde_json::Value,
}

impl Continue {
    /// 创建继续状态
    pub fn new(reason: ContinueReason) -> Self {
        Self {
            reason,
            metadata: serde_json::Value::Null,
        }
    }

    /// 带元数据的继续状态
    pub fn with_metadata(reason: ContinueReason, metadata: serde_json::Value) -> Self {
        Self { reason, metadata }
    }
}

/// 流式请求事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRequestStart {
    /// 请求 ID
    pub request_id: String,
    /// 轮次计数
    pub turn_count: usize,
}

impl StreamRequestStart {
    pub fn new(request_id: impl Into<String>, turn_count: usize) -> Self {
        Self {
            request_id: request_id.into(),
            turn_count,
        }
    }
}

/// 流式事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// 内容块增量
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: ContentDelta },
    /// 内容块开始
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: serde_json::Value,
    },
    /// 内容块结束
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    /// 响应元数据
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDelta,
        usage: UsageInfo,
    },
    /// 消息开始
    #[serde(rename = "message_start")]
    MessageStart { message: serde_json::Value },
    /// 消息结束
    #[serde(rename = "message_stop")]
    MessageStop,
    /// 进度消息
    #[serde(rename = "progress")]
    Progress { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub output_tokens: usize,
    pub input_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::UserMessage;

    #[test]
    fn test_state_creation() {
        let messages = vec![Message::User(UserMessage::text("Hello"))];
        let state = State::initial(messages);
        assert_eq!(state.turn_count, 0);
        assert_eq!(state.output_token_recovery_count, 0);
    }

    #[test]
    fn test_state_transition() {
        let messages = vec![Message::User(UserMessage::text("Hello"))];
        let state = State::initial(messages.clone());

        let next_state = state.next(ContinueReason::NextTurn, messages.clone());

        assert_eq!(next_state.turn_count, 1);
        assert!(matches!(
            next_state.transition.reason,
            Some(ContinueReason::NextTurn)
        ));
    }

    #[test]
    fn test_terminal_creation() {
        let terminal = Terminal::new(TerminalReason::Completed);
        assert!(terminal.message.is_none());

        let terminal = Terminal::with_message(TerminalReason::ModelError, "API call failed");
        assert!(terminal.message.is_some());
    }

    #[test]
    fn test_continue_creation() {
        let continue_state = Continue::new(ContinueReason::NextTurn);
        assert!(matches!(continue_state.reason, ContinueReason::NextTurn));
    }
}
