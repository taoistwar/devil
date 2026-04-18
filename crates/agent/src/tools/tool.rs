//! 工具核心定义模块
//!
//! 基于 Claude Code 架构定义 Tool 五要素协议：
//! 1. 名称与别名 - 唯一标识符 + 可选别名
//! 2. Schema - 运行时验证 + API 通信
//! 3. 权限模型 - 三层分层检查
//! 4. 执行逻辑 - 核心执行方法
//! 5. UI 渲染 - 完整生命周期渲染

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pub file_cache: Arc<RwLock<HashMap<String, FileState>>>,
    /// 工具结果目录
    pub tool_results_dir: Option<String>,
}

/// 文件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 中断行为
///
/// 定义用户提交新消息时工具的行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterruptBehavior {
    /// 停止工具并丢弃结果
    Cancel,
    /// 继续运行，新消息等待
    Block,
}

impl Default for InterruptBehavior {
    fn default() -> Self {
        Self::Block
    }
}

/// 权限行为
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior", rename_all = "lowercase")]
pub enum PermissionBehavior {
    /// 允许执行
    Allow {
        /// 更新后的输入（可选）
        #[serde(rename = "updatedInput")]
        updated_input: Option<serde_json::Value>,
    },
    /// 拒绝
    Deny {
        /// 拒绝原因
        reason: String,
    },
    /// 询问用户
    Ask {
        /// 提示内容
        prompt: String,
    },
}

/// 权限检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResult {
    /// 权限行为
    pub behavior: PermissionBehavior,
}

impl PermissionResult {
    pub fn allow() -> Self {
        Self {
            behavior: PermissionBehavior::Allow {
                updated_input: None,
            },
        }
    }

    pub fn allow_with_input(updated_input: serde_json::Value) -> Self {
        Self {
            behavior: PermissionBehavior::Allow {
                updated_input: Some(updated_input),
            },
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            behavior: PermissionBehavior::Deny {
                reason: reason.into(),
            },
        }
    }

    pub fn ask(prompt: impl Into<String>) -> Self {
        Self {
            behavior: PermissionBehavior::Ask {
                prompt: prompt.into(),
            },
        }
    }
}

/// 输入验证结果
#[derive(Debug, Clone)]
pub struct InputValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 错误消息（如果无效）
    pub error_message: Option<String>,
    /// 错误代码
    pub error_code: Option<i32>,
}

impl InputValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            error_message: None,
            error_code: None,
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            error_message: Some(message.into()),
            error_code: None,
        }
    }

    pub fn invalid_with_code(message: impl Into<String>, code: i32) -> Self {
        Self {
            is_valid: false,
            error_message: Some(message.into()),
            error_code: Some(code),
        }
    }
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult<O> {
    /// 工具调用 ID
    pub tool_use_id: String,
    /// 是否执行成功
    pub is_success: bool,
    /// 输出数据
    pub output: Option<O>,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 上下文修改器（可选）
    pub context_modifier: Option<ContextModifier>,
    /// 是否被中断
    pub interrupted: bool,
}

impl<O> ToolResult<O> {
    /// 创建成功的结果
    pub fn success(tool_use_id: impl Into<String>, output: O) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            is_success: true,
            output: Some(output),
            error: None,
            context_modifier: None,
            interrupted: false,
        }
    }

    /// 创建失败的结果
    pub fn error(
        tool_use_id: impl Into<String>,
        error: impl Into<String>,
    ) -> ToolResult<serde_json::Value> {
        ToolResult {
            tool_use_id: tool_use_id.into(),
            is_success: false,
            output: None,
            error: Some(error.into()),
            context_modifier: None,
            interrupted: false,
        }
    }

    /// 创建中断的结果
    pub fn interrupted(tool_use_id: impl Into<String>) -> ToolResult<serde_json::Value> {
        ToolResult {
            tool_use_id: tool_use_id.into(),
            is_success: false,
            output: None,
            error: Some("Interrupted by user".to_string()),
            context_modifier: None,
            interrupted: true,
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

/// 工具进度数据 Trait
pub trait ToolProgressData: Clone + Send + Sync {}

impl ToolProgressData for serde_json::Value {}
impl ToolProgressData for String {}

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

/// 搜索/读取命令分析结果
#[derive(Debug, Clone, Default)]
pub struct SearchOrReadResult {
    /// 是否为搜索操作 (grep, find, glob patterns)
    pub is_search: bool,
    /// 是否为读取操作 (cat, head, tail, file read)
    pub is_read: bool,
    /// 是否为列表操作 (ls, tree, du)
    pub is_list: bool,
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

    // ===== 要素一：名称与标识 =====

    /// 获取工具唯一名称
    fn name(&self) -> &str;

    /// 获取工具描述
    fn description(&self) -> &str {
        ""
    }

    /// 获取工具别名（用于向后兼容）
    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// 获取用户可见名称
    fn user_facing_name(&self, _input: Option<&Self::Input>) -> String {
        self.name().to_string()
    }

    // ===== 要素二：Schema =====

    /// 获取输入参数 Schema（JSON Schema 格式）
    fn input_schema(&self) -> serde_json::Value;

    /// 验证输入参数
    fn validate_input(&self, input: &serde_json::Value) -> InputValidationResult {
        // 默认实现：总是有效
        InputValidationResult::valid()
    }

    // ===== 要素三：权限模型 =====

    /// 第一层：输入验证
    /// 在权限检查之前运行，用于拒绝无效输入
    fn validate_input_permissions(
        &self,
        input: &Self::Input,
        _context: &ToolContext,
    ) -> InputValidationResult {
        InputValidationResult::valid()
    }

    /// 第二层：权限检查
    /// 检查是否有权限使用此工具
    async fn check_permissions(
        &self,
        input: &Self::Input,
        _context: &ToolContext,
    ) -> PermissionResult {
        // 默认：允许执行
        PermissionResult::allow()
    }

    /// 获取权限级别
    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    // ===== 要素四：运行时属性 =====

    /// 工具是否启用
    ///
    /// 用于功能开关控制，支持编译期死代码消除
    fn is_enabled(&self) -> bool {
        true
    }

    /// 判断工具是否为只读操作
    fn is_read_only(&self) -> bool {
        false
    }

    /// 判断工具是否支持并发执行
    ///
    /// fail-closed 原则：默认为 false，工具必须显式声明自己安全
    fn is_concurrency_safe(&self) -> bool {
        false
    }

    /// 判断工具是否为破坏性操作
    ///
    /// 仅当工具执行不可逆操作时返回 true（删除、覆盖、发送）
    fn is_destructive(&self, _input: &Self::Input) -> bool {
        false
    }

    /// 判断此工具是否为搜索或读取操作
    ///
    /// 用于 UI 折叠展示
    fn is_search_or_read_command(&self, _input: &Self::Input) -> SearchOrReadResult {
        SearchOrReadResult::default()
    }

    /// 获取超时时间（毫秒）
    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        None
    }

    /// 获取最大结果大小（字符数）
    ///
    /// 超过此值时结果将被持久化到文件
    fn max_result_size_chars(&self) -> usize {
        100_000 // 默认 100KB
    }

    /// 是否在延迟发现模式下始终加载
    fn should_always_load(&self) -> bool {
        false
    }

    /// 是否应该延迟加载工具 schema
    fn should_defer(&self) -> bool {
        false
    }

    /// 中断行为
    ///
    /// 定义用户提交新消息时工具的行为
    fn interrupt_behavior(&self) -> InterruptBehavior {
        InterruptBehavior::Block
    }

    /// 是否为透明包装器
    ///
    /// 透明包装器（如 REPL）将所有渲染委托给进度处理器
    fn is_transparent_wrapper(&self) -> bool {
        false
    }

    /// 是否为开放世界工具（可能访问外部资源）
    fn is_open_world(&self, _input: &Self::Input) -> bool {
        false
    }

    /// 是否需要用户交互
    fn requires_user_interaction(&self) -> bool {
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

    /// 工具调用开始时的展示消息
    fn render_use_message(&self, input: &Self::Input) -> String {
        format!("Using {}...", self.user_facing_name(Some(input)))
    }

    /// 获取活动描述（用于 spinner 显示）
    ///
    /// 示例："Reading src/foo.ts", "Running bun test"
    fn get_activity_description(&self, _input: &Self::Input) -> Option<String> {
        None
    }

    /// 工具执行中的进度展示
    fn render_progress_message(&self, progress: &ToolProgress<Self::Progress>) -> Option<String> {
        progress.message.clone()
    }

    /// 工具结果展示
    fn render_result_message(&self, result: &ToolResult<Self::Output>) -> String {
        if result.is_success {
            if result.interrupted {
                "Operation interrupted".to_string()
            } else {
                "Operation completed successfully".to_string()
            }
        } else {
            format!(
                "Operation failed: {}",
                result.error.as_deref().unwrap_or("unknown error")
            )
        }
    }

    /// 获取工具使用摘要（用于紧凑视图）
    fn get_tool_use_summary(&self, _input: &Self::Input) -> Option<String> {
        None
    }

    /// 权限被拒绝时的展示
    fn render_rejected_message(&self, reason: &str) -> String {
        format!("Tool {} was rejected: {}", self.name(), reason)
    }

    /// 执行出错时的展示
    fn render_error_message(&self, error: &str) -> String {
        format!("Tool {} error: {}", self.name(), error)
    }

    /// 准备权限匹配器（用于 hook 条件）
    fn prepare_permission_matcher(
        &self,
        _input: &Self::Input,
    ) -> Option<Box<dyn Fn(&str) -> bool + Send + Sync>> {
        None
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

impl Default for ToolPermissionLevel {
    fn default() -> Self {
        Self::ReadOnly
    }
}

/// 工具元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub permission_level: ToolPermissionLevel,
    pub concurrency_safe: bool,
    pub read_only: bool,
    pub timeout_secs: Option<u64>,
    pub always_load: bool,
    pub aliases: Vec<String>,
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
    pub fn new(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
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

/// 延迟 Schema 评估器
///
/// 用于延迟评估工具 schema，避免不必要的计算
pub struct LazySchema<T> {
    schema_fn: Arc<dyn Fn() -> T + Send + Sync>,
    cached_value: Arc<RwLock<Option<T>>>,
}

impl<T: Clone> LazySchema<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            schema_fn: Arc::new(f),
            cached_value: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get(&self) -> T {
        // 先尝试读取缓存
        {
            let cached = self.cached_value.read().await;
            if let Some(value) = cached.as_ref() {
                return value.clone();
            }
        }

        // 缓存未命中，计算并缓存
        let value = (self.schema_fn)();
        let mut cached = self.cached_value.write().await;
        *cached = Some(value.clone());
        value
    }
}

impl<T: Clone> Clone for LazySchema<T> {
    fn clone(&self) -> Self {
        Self {
            schema_fn: self.schema_fn.clone(),
            cached_value: self.cached_value.clone(),
        }
    }
}

/// 创建延迟 schema 的辅助函数
pub fn lazy_schema<T, F>(f: F) -> LazySchema<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    LazySchema::new(f)
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
        type Progress = String;

        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "A test tool"
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

        fn interrupt_behavior(&self) -> InterruptBehavior {
            InterruptBehavior::Cancel
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
        assert_eq!(tool.interrupt_behavior(), InterruptBehavior::Cancel);
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let tool = TestTool;
        let ctx = ToolContext::default();
        let input = serde_json::json!({ "message": "Hello" });

        let result = tool
            .execute(input, &ctx, None::<fn(ToolProgress<String>)>)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_permission_result() {
        let allow = PermissionResult::allow();
        assert!(matches!(allow.behavior, PermissionBehavior::Allow { .. }));

        let deny = PermissionResult::deny("Access denied");
        assert!(matches!(deny.behavior, PermissionBehavior::Deny { .. }));

        let ask = PermissionResult::ask("Confirm?");
        assert!(matches!(ask.behavior, PermissionBehavior::Ask { .. }));
    }

    #[test]
    fn test_input_validation_result() {
        let valid = InputValidationResult::valid();
        assert!(valid.is_valid);

        let invalid = InputValidationResult::invalid("Invalid input");
        assert!(!invalid.is_valid);
        assert_eq!(invalid.error_message, Some("Invalid input".to_string()));
    }

    #[tokio::test]
    async fn test_lazy_schema() {
        let call_count = Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        let lazy = lazy_schema(move || {
            *call_count_clone.lock().unwrap() += 1;
            "schema_value".to_string()
        });

        // 第一次调用应该计算
        let value1 = lazy.get().await;
        assert_eq!(value1, "schema_value");
        assert_eq!(*call_count.lock().unwrap(), 1);

        // 第二次调用应该使用缓存
        let value2 = lazy.get().await;
        assert_eq!(value2, "schema_value");
        assert_eq!(*call_count.lock().unwrap(), 1); // 仍然是 1
    }
}
