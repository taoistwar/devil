//! 工具核心定义模块
//! 
//! 定义 Tool 五要素协议：
//! 1. 名称与别名 - 唯一标识符 + 可选别名
//! 2. Schema - 运行时验证 + API 通信
//! 3. 权限模型 - 三层分层检查
//! 4. 执行逻辑 - 核心执行方法
//! 5. UI 渲染 - 完整生命周期渲染

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工具执行上下文
/// 
/// 包含工具执行所需的环境信息
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// 当前工作目录
    pub working_directory: Option<String>,
    /// 环境变量
    pub env: HashMap<String, String>,
    /// 已执行的工具历史记录
    pub executed_tools: Vec<ToolExecutionRecord>,
    /// 文件状态缓存
    pub file_cache: HashMap<String, FileState>,
}

/// 文件状态
#[derive(Debug, Clone)]
pub struct FileState {
    /// 文件路径
    pub path: String,
    /// 是否已读取
    pub has_been_read: bool,
    /// 最后修改时间戳
    pub last_modified: Option<u64>,
    /// 内容哈希
    pub content_hash: Option<String>,
}

/// 工具执行记录
#[derive(Debug, Clone)]
pub struct ToolExecutionRecord {
    /// 工具调用 ID
    pub tool_use_id: String,
    /// 工具名称
    pub tool_name: String,
    /// 执行是否成功
    pub success: bool,
    /// 执行耗时（毫秒）
    pub duration_ms: Option<u64>,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult<O> {
    /// 工具调用 ID
    pub tool_use_id: String,
    /// 是否执行成功
    pub is_success: bool,
    /// 输出数据
    pub output: O,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 上下文修改器（可选）
    pub context_modifier: Option<ContextModifier>,
}

impl<O> ToolResult<O> {
    /// 创建成功的结果
    pub fn success(tool_use_id: impl Into<String>, output: O) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            is_success: true,
            output,
            error: None,
            context_modifier: None,
        }
    }

    /// 创建失败的结果
    pub fn error(
        tool_use_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            is_success: false,
            output: serde_json::Value::Null,
            error: Some(error.into()),
            context_modifier: None,
        }
    }

    /// 添加上下文修改器
    pub fn with_context_modifier(mut self, modifier: ContextModifier) -> Self {
        self.context_modifier = Some(modifier);
        self
    }
}

/// 上下文修改器
/// 
/// 允许工具在执行后修改上下文（如更新文件缓存）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextModifier {
    /// 要更新的文件状态
    pub file_updates: Vec<FileState>,
    /// 其他上下文更新
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ContextModifier {
    pub fn new() -> Self {
        Self {
            file_updates: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for ContextModifier {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具权限级别
/// 
/// 分层权限检查的第二层
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermissionLevel {
    /// 只读操作，无需确认
    ReadOnly,
    /// 可能产生副作用，需要确认
    RequiresConfirmation,
    /// 破坏性操作，严格受限
    Destructive,
    /// 全局拒绝（任何情况下都不允许）
    BlanketDenied,
}

/// 工具元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入参数 Schema
    pub input_schema: serde_json::Value,
    /// 权限级别
    pub permission_level: ToolPermissionLevel,
    /// 是否支持并发执行
    pub concurrency_safe: bool,
    /// 是否为只读操作
    pub read_only: bool,
    /// 超时时间（秒）
    pub timeout_secs: Option<u64>,
    /// 是否在延迟发现模式下始终加载
    pub always_load: bool,
    /// 别名列表（用于向后兼容）
    pub aliases: Vec<String>,
}

/// 工具输入验证结果
#[derive(Debug)]
pub struct InputValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 错误消息（如果无效）
    pub error_message: Option<String>,
}

/// 权限检查结果
#[derive(Debug)]
pub struct PermissionCheckResult {
    /// 是否有权限
    pub has_permission: bool,
    /// 拒绝原因（如果没有权限）
    pub denial_reason: Option<String>,
    /// 是否需要用户确认
    pub requires_confirmation: bool,
}

/// 工具进度数据 Trait
pub trait ToolProgressData: Send + Sync {}

/// 工具进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProgress<P: ToolProgressData> {
    /// 进度数据
    pub data: P,
    /// 进度百分比（0-100）
    pub percentage: Option<u8>,
    /// 进度消息
    pub message: Option<String>,
}

/// Tool 五要素协议 Trait
/// 
/// 每个工具必须实现的完整接口
/// - Input: 使用 schema 定义的结构化输入
/// - Output: 工具输出类型
/// - P: 进度数据类型
#[async_trait]
pub trait Tool: Send + Sync {
    /// 输入类型
    type Input: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    /// 输出类型
    type Output: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    /// 进度数据类型
    type Progress: ToolProgressData;

    // ===== 要素一：名称与别名 =====

    /// 获取工具唯一名称
    fn name(&self) -> &str;

    /// 获取工具别名（用于向后兼容）
    fn aliases(&self) -> &[&str] {
        &[]
    }

    // ===== 要素二：Schema =====

    /// 获取输入参数 Schema（JSON Schema 格式）
    fn input_schema(&self) -> serde_json::Value;

    /// 验证输入参数
    fn validate_input(&self, input: &serde_json::Value) -> InputValidationResult {
        // 默认实现：总是有效
        // 子类可以重写以添加自定义验证
        InputValidationResult {
            is_valid: true,
            error_message: None,
        }
    }

    // ===== 要素三：权限模型 =====

    /// 第一层：输入验证
    /// 在权限检查之前运行，用于拒绝无效输入
    fn validate_input_permissions(
        &self,
        input: &Self::Input,
    ) -> InputValidationResult {
        InputValidationResult {
            is_valid: true,
            error_message: None,
        }
    }

    /// 第二层：权限检查
    /// 检查是否有权限使用此工具
    fn has_permission(&self, input: &Self::Input, ctx: &ToolContext) -> bool {
        true
    }

    /// 检查权限，返回详细信息
    fn check_permissions(
        &self,
        input: &Self::Input,
        ctx: &ToolContext,
    ) -> PermissionCheckResult {
        let has_permission = self.has_permission(input, ctx);
        PermissionCheckResult {
            has_permission,
            denial_reason: if has_permission {
                None
            } else {
                Some("Permission denied".to_string())
            },
            requires_confirmation: !has_permission,
        }
    }

    /// 获取权限级别
    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    // ===== 要素四：运行时属性 =====

    /// 判断工具是否为只读操作
    fn is_read_only(&self) -> bool {
        false
    }

    /// 判断工具是否支持并发执行
    /// 
    /// 这是并发分区策略的关键判断依据
    /// 
    /// fail-closed 原则：默认为 false，工具必须显式声明自己安全
    fn is_concurrency_safe(&self) -> bool {
        false
    }

    /// 判断工具是否为破坏性操作
    fn is_destructive(&self, input: &Self::Input) -> bool {
        false
    }

    /// 获取超时时间（秒）
    fn timeout_secs(&self) -> Option<u64> {
        None
    }

    /// 是否在延迟发现模式下始终加载
    fn should_always_load(&self) -> bool {
        false
    }

    // ===== 要素五：执行逻辑 =====

    /// 核心执行方法
    /// 
    /// 接收解析后的输入参数、工具执行上下文、进度回调
    /// 返回结果携带输出数据和可选的上下文修改器
    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>>;

    // ===== 要素六：UI 渲染 =====
    // 注意：Rust 中 UI 渲染简化为字符串模板

    /// 工具调用开始时的展示消息
    fn render_use_message(&self, input: &Self::Input) -> String {
        format!("Using {}...", self.name())
    }

    /// 工具执行中的进度展示
    fn render_progress_message(&self, progress: &ToolProgress<Self::Progress>) -> Option<String> {
        progress.message.clone()
    }

    /// 工具结果展示
    fn render_result_message(&self, result: &ToolResult<Self::Output>) -> String {
        if result.is_success {
            "Operation completed successfully".to_string()
        } else {
            format!("Operation failed: {}", result.error.as_deref().unwrap_or("unknown error"))
        }
    }

    /// 权限被拒绝时的展示
    fn render_rejected_message(&self, reason: &str) -> String {
        format!("Tool {} was rejected: {}", self.name(), reason)
    }

    /// 执行出错时的展示
    fn render_error_message(&self, error: &str) -> String {
        format!("Tool {} error: {}", self.name(), error)
    }
}

/// 工具调用块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// 工具调用 ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 输入参数
    pub input: serde_json::Value,
}

impl ToolUseBlock {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
        }
    }
}

/// 工具使用块（带进度追踪）
#[derive(Debug)]
pub struct ToolUseBlockWithProgress {
    /// 工具调用块
    pub block: ToolUseBlock,
    /// 是否是并发安全工具
    pub concurrency_safe: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // 简单的测试工具
    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        type Input = serde_json::Value;
        type Output = serde_json::Value;
        type Progress = serde_json::Value;

        fn name(&self) -> &str {
            "test"
        }

        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                }
            })
        }

        fn is_concurrency_safe(&self) -> bool {
            true
        }

        fn is_read_only(&self) -> bool {
            true
        }

        async fn execute(
            &self,
            input: Self::Input,
            _ctx: &ToolContext,
            _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
        ) -> Result<ToolResult<Self::Output>> {
            Ok(ToolResult::success("test-1", input))
        }
    }

    #[test]
    fn test_tool_metadata() {
        let tool = TestTool;
        assert_eq!(tool.name(), "test");
        assert!(tool.is_concurrency_safe());
        assert!(tool.is_read_only());
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let tool = TestTool;
        let ctx = ToolContext::default();
        let input = serde_json::json!({ "message": "Hello" });

        let result = tool.execute(input, &ctx, None::<fn(ToolProgress<serde_json::Value>)>).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_context_modifier() {
        let modifier = ContextModifier::new();
        assert!(modifier.file_updates.is_empty());
        assert!(modifier.metadata.is_empty());
    }
}
