//! 消息类型模块
//! 
//! 定义 Agent 对话中使用的消息类型，包括：
//! - UserMessage: 用户输入和工具执行结果
//! - AssistantMessage: 助手回复，可包含工具调用
//! - SystemMessage: 系统级通知
//! - ToolUseSummaryMessage: 工具调用摘要

use serde::{Deserialize, Serialize};

/// 消息内容块类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// 文本内容
    #[serde(rename = "text")]
    Text { text: String },
    
    /// 工具调用块
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    
    /// 工具执行结果
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    
    /// Thinking 块（模型的思考过程）
    #[serde(rename = "thinking")]
    Thinking { text: String },
}

impl ContentBlock {
    pub fn text_len(&self) -> usize {
        match self {
            ContentBlock::Text { text } => text.len(),
            ContentBlock::ToolUse { input, .. } => input.to_string().len(),
            ContentBlock::ToolResult { content, .. } => content.len(),
            ContentBlock::Thinking { text } => text.len(),
        }
    }
}

/// 基础消息 Trait
pub trait BaseMessage {
    /// 获取消息角色
    fn role(&self) -> MessageRole;
    
    /// 获取消息内容
    fn content(&self) -> &[ContentBlock];
}

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// 用户消息
/// 承载用户输入和工具执行结果（tool_result）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub content: Vec<ContentBlock>,
}

impl UserMessage {
    /// 创建纯文本用户消息
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    /// 添加工具执行结果
    pub fn with_tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self {
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: content.into(),
                is_error,
            }],
        }
    }
}

impl BaseMessage for UserMessage {
    fn role(&self) -> MessageRole {
        MessageRole::User
    }

    fn content(&self) -> &[ContentBlock] {
        &self.content
    }
}

/// 助手消息
/// 可同时包含文本和工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub content: Vec<ContentBlock>,
}

impl AssistantMessage {
    /// 创建纯文本助手消息
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    /// 添加工具调用
    pub fn with_tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self {
            content: vec![ContentBlock::ToolUse {
                id: id.into(),
                name: name.into(),
                input,
            }],
        }
    }

    /// 获取所有工具调用块
    pub fn tool_use_blocks(&self) -> Vec<&ContentBlock> {
        self.content
            .iter()
            .filter(|c| matches!(c, ContentBlock::ToolUse { .. }))
            .collect()
    }

    /// 获取纯文本内容（不包括工具调用）
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| match c {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

impl BaseMessage for AssistantMessage {
    fn role(&self) -> MessageRole {
        MessageRole::Assistant
    }

    fn content(&self) -> &[ContentBlock] {
        &self.content
    }
}

/// 系统消息
/// 用于系统级通知，如权限变更、模型回退提示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub content: String,
    pub level: SystemMessageLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemMessageLevel {
    Info,
    Warning,
    Error,
}

impl SystemMessage {
    /// 创建信息级系统消息
    pub fn info(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            level: SystemMessageLevel::Info,
        }
    }

    /// 创建警告级系统消息
    pub fn warning(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            level: SystemMessageLevel::Warning,
        }
    }

    /// 创建错误级系统消息
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            level: SystemMessageLevel::Error,
        }
    }
}

/// 工具调用摘要消息
/// 在一批工具执行完成后，用于在 UI 中折叠展示工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseSummaryMessage {
    /// 摘要内容
    pub summary: String,
    /// 参与的工具数量
    pub tool_count: usize,
    /// 执行状态
    pub execution_status: ExecutionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Success,
    PartialSuccess,
    Failed,
}

impl ToolUseSummaryMessage {
    /// 创建工具调用摘要
    pub fn new(summary: impl Into<String>, tool_count: usize, status: ExecutionStatus) -> Self {
        Self {
            summary: summary.into(),
            tool_count,
            execution_status: status,
        }
    }
}

/// 墓碑消息
/// 当流式回退发生时，标记已废弃的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TombstoneMessage {
    /// 被废弃消息的 ID
    pub message_id: String,
    /// 原因
    pub reason: String,
}

impl TombstoneMessage {
    pub fn new(message_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            reason: reason.into(),
        }
    }
}

/// 附件消息
/// 承载文件变更通知、内存文件内容、任务通知等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMessage {
    pub attachment_type: AttachmentType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentType {
    FileChange,
    MemoryFile,
    TaskNotification,
}

/// 进度消息
/// 用于实时反馈工具运行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMessage {
    pub tool_name: String,
    pub progress: String,
    pub is_complete: bool,
}

/// 统一的消息枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    #[serde(skip)]
    System(SystemMessage),
    #[serde(rename = "tool_summary")]
    ToolSummary(ToolUseSummaryMessage),
    #[serde(skip)]
    Tombstone(TombstoneMessage),
    #[serde(rename = "attachment")]
    Attachment(AttachmentMessage),
    #[serde(rename = "progress")]
    Progress(ProgressMessage),
}

impl Message {
    /// 创建用户文本消息
    pub fn user_text(text: impl Into<String>) -> Self {
        Self::User(UserMessage::text(text))
    }

    /// 创建助手文本消息
    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self::Assistant(AssistantMessage::text(text))
    }

    /// 获取消息角色
    pub fn role(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::Assistant(_) => "assistant",
            Self::System(_) => "system",
            Self::ToolSummary(_) => "tool_summary",
            Self::Tombstone(_) => "tombstone",
            Self::Attachment(_) => "attachment",
            Self::Progress(_) => "progress",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let msg = UserMessage::text("Hello, world!");
        assert_eq!(msg.role(), MessageRole::User);
        assert_eq!(msg.content.len(), 1);
    }

    #[test]
    fn test_assistant_message_with_tool_use() {
        let msg = AssistantMessage::with_tool_use(
            "tool-1",
            "read_file",
            serde_json::json!({"path": "test.txt"}),
        );
        assert_eq!(msg.role(), MessageRole::Assistant);
        assert_eq!(msg.tool_use_blocks().len(), 1);
    }

    #[test]
    fn test_system_message() {
        let msg = SystemMessage::warning("Token budget warning");
        assert_eq!(msg.level, SystemMessageLevel::Warning);
    }

    #[test]
    fn test_tool_summary_message() {
        let msg = ToolUseSummaryMessage::new(
            "Executed 3 tools successfully",
            3,
            ExecutionStatus::Success,
        );
        assert_eq!(msg.tool_count, 3);
    }
}
