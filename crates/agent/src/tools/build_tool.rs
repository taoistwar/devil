//! buildTool 工厂函数模块
//! 
//! 基于 Claude Code 的 buildTool 实现，提供：
//! - 安全默认值（fail-closed）
//! - 类型级合并（部分字段可选）
//! - 统一的工具构建入口

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::tools::tool::{
    Tool, ToolContext, ToolResult, ToolPermissionLevel,
    InputValidationResult, PermissionResult, ContextModifier,
    ToolProgress, ToolProgressData, InterruptBehavior, SearchOrReadResult,
};

/// buildTool 工厂函数
/// 
/// 创建工具的标准工厂函数，自动填充安全默认值
/// 
/// 遵循 "fail-closed" 原则：
/// 安全性相关的方法（如并发安全判断、只读判断）默认为 false
/// 工具必须显式声明自己安全才能享受并发等优化
/// 
/// # 默认值
/// - `is_enabled` → `true`
/// - `is_concurrency_safe` → `false` (assume not safe)
/// - `is_read_only` → `false` (assume writes)
/// - `is_destructive` → `false`
/// - `check_permissions` → `PermissionResult::allow()` (defer to general permission system)
/// - `interrupt_behavior` → `InterruptBehavior::Block`
/// - `user_facing_name` → `name`
pub struct ToolBuilder<I, O, P = String> {
    // 必填字段
    name: String,
    description: String,
    input_schema: serde_json::Value,
    
    // 可选字段（有默认值）
    aliases: Vec<String>,
    permission_level: ToolPermissionLevel,
    concurrency_safe: bool,
    read_only: bool,
    destructive: bool,
    timeout_ms: Option<u64>,
    max_result_size_chars: usize,
    always_load: bool,
    should_defer: bool,
    interrupt_behavior: InterruptBehavior,
    transparent_wrapper: bool,
    
    // 执行函数
    execute_fn: Option<Box<dyn Fn(I, &ToolContext, Option<tokio::sync::watch::Receiver<bool>>) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<O>> + Send>>>>,
    
    // 虚拟类型参数
    _phantom: PhantomData<(I, O, P)>,
}

impl<I, O, P> ToolBuilder<I, O, P>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
    P: ToolProgressData + 'static,
{
    /// 创建新的工具构建器
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: serde_json::Value::Null,
            aliases: Vec::new(),
            permission_level: ToolPermissionLevel::RequiresConfirmation,
            concurrency_safe: false, // fail-closed: 默认不安全
            read_only: false,        // fail-closed: 默认不是只读
            destructive: false,      // fail-closed: 默认不是破坏性
            timeout_ms: None,
            max_result_size_chars: 100_000, // 默认 100KB
            always_load: false,
            should_defer: false,
            interrupt_behavior: InterruptBehavior::Block,
            transparent_wrapper: false,
            execute_fn: None,
            _phantom: PhantomData,
        }
    }

    // ===== Schema 配置 =====

    /// 设置输入 schema
    pub fn input_schema(mut self, schema: serde_json::Value) -> Self {
        self.input_schema = schema;
        self
    }

    /// 设置别名
    pub fn aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    // ===== 权限配置 =====

    /// 设置权限级别
    pub fn permission_level(mut self, level: ToolPermissionLevel) -> Self {
        self.permission_level = level;
        self
    }

    /// 标记为只读
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self.permission_level = ToolPermissionLevel::ReadOnly;
        self
    }

    // ===== 运行时属性配置 =====

    /// 标记为并发安全
    pub fn concurrency_safe(mut self) -> Self {
        self.concurrency_safe = true;
        self
    }

    /// 标记为破坏性操作
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self.permission_level = ToolPermissionLevel::Destructive;
        self
    }

    /// 设置超时时间（毫秒）
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// 设置最大结果大小（字符数）
    pub fn max_result_size_chars(mut self, size: usize) -> Self {
        self.max_result_size_chars = size;
        self
    }

    /// 标记为始终加载（用于延迟发现）
    pub fn always_load(mut self) -> Self {
        self.always_load = true;
        self
    }

    /// 标记为延迟加载
    pub fn should_defer(mut self) -> Self {
        self.should_defer = true;
        self
    }

    /// 设置中断行为
    pub fn interrupt_behavior(mut self, behavior: InterruptBehavior) -> Self {
        self.interrupt_behavior = behavior;
        self
    }

    /// 标记为透明包装器
    pub fn transparent_wrapper(mut self) -> Self {
        self.transparent_wrapper = true;
        self
    }

    // ===== 执行函数配置 =====

    /// 设置执行函数
    pub fn execute<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(I, &ToolContext, Option<tokio::sync::watch::Receiver<bool>>) -> Fut 
           + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<O>> + Send + 'static,
    {
        self.execute_fn = Some(Box::new(move |input, ctx, cancel| {
            Box::pin(f(input, ctx, cancel))
        }));
        self
    }

    /// 设置执行函数（简化版，无需 cancel signal）
    pub fn execute_simple<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(I, &ToolContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<O>> + Send + 'static,
    {
        self.execute_fn = Some(Box::new(move |input, ctx, _| {
            Box::pin(f(input, ctx))
        }));
        self
    }

    /// 构建工具
    pub fn build(self) -> BuiltTool<I, O, P> {
        BuiltTool {
            name: self.name,
            description: self.description,
            input_schema: self.input_schema,
            aliases: self.aliases,
            permission_level: self.permission_level,
            concurrency_safe: self.concurrency_safe,
            read_only: self.read_only,
            destructive: self.destructive,
            timeout_ms: self.timeout_ms,
            max_result_size_chars: self.max_result_size_chars,
            always_load: self.always_load,
            should_defer: self.should_defer,
            interrupt_behavior: self.interrupt_behavior,
            transparent_wrapper: self.transparent_wrapper,
            execute_fn: self.execute_fn.unwrap_or_else(|| {
                Box::new(|_, _, _| {
                    Box::pin(async {
                        anyhow::bail!("No execute function provided")
                    })
                })
            }),
            _phantom: PhantomData,
        }
    }
}

/// 构建完成的工具
pub struct BuiltTool<I, O, P = String> {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    aliases: Vec<String>,
    permission_level: ToolPermissionLevel,
    concurrency_safe: bool,
    read_only: bool,
    destructive: bool,
    timeout_ms: Option<u64>,
    max_result_size_chars: usize,
    always_load: bool,
    should_defer: bool,
    interrupt_behavior: InterruptBehavior,
    transparent_wrapper: bool,
    execute_fn: Box<dyn Fn(I, &ToolContext, Option<tokio::sync::watch::Receiver<bool>>) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<O>> + Send>> + Send + Sync>,
    _phantom: PhantomData<(I, O, P)>,
}

#[async_trait::async_trait]
impl<I, O, P> Tool for BuiltTool<I, O, P>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    P: ToolProgressData,
{
    type Input = I;
    type Output = O;
    type Progress = P;

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn aliases(&self) -> &[&str] {
        self.aliases.iter().map(|s| s.as_str()).collect::<Vec<_>>().leak()
    }

    fn user_facing_name(&self, _input: Option<&Self::Input>) -> String {
        self.name.clone()
    }

    fn input_schema(&self) -> serde_json::Value {
        self.input_schema.clone()
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        self.permission_level
    }

    fn is_enabled(&self) -> bool {
        true // 默认启用，可通过外部包装控制
    }

    fn is_read_only(&self, _input: &Self::Input) -> bool {
        self.read_only
    }

    fn is_concurrency_safe(&self, _input: &Self::Input) -> bool {
        self.concurrency_safe
    }

    fn is_destructive(&self, _input: &Self::Input) -> bool {
        self.destructive
    }

    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        self.timeout_ms
    }

    fn max_result_size_chars(&self) -> usize {
        self.max_result_size_chars
    }

    fn should_always_load(&self) -> bool {
        self.always_load
    }

    fn should_defer(&self) -> bool {
        self.should_defer
    }

    fn interrupt_behavior(&self) -> InterruptBehavior {
        self.interrupt_behavior
    }

    fn is_transparent_wrapper(&self) -> bool {
        self.transparent_wrapper
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
        cancel_signal: Option<tokio::sync::watch::Receiver<bool>>,
    ) -> Result<ToolResult<Self::Output>, anyhow::Error> {
        // 简化处理，忽略 progress_callback
        let _ = progress_callback;
        let output = (self.execute_fn)(input, ctx, cancel_signal).await?;
        let tool_use_id = Uuid::new_v4().to_string();
        Ok(ToolResult::success(tool_use_id, output))
    }
}

// ===== 工具结果持久化辅助函数 =====

/// 确保工具结果目录存在
pub fn ensure_tool_results_dir(base_dir: &str) -> std::io::Result<String> {
    let tool_results_dir = std::path::Path::new(base_dir).join("tool-results");
    std::fs::create_dir_all(&tool_results_dir)?;
    Ok(tool_results_dir.to_string_lossy().to_string())
}

/// 生成工具结果文件路径
pub fn get_tool_result_path(tool_results_dir: &str, tool_use_id: &str) -> String {
    std::path::Path::new(tool_results_dir)
        .join(format!("{}.json", tool_use_id))
        .to_string_lossy()
        .to_string()
}

/// 当工具结果超过大小时，生成预览
pub fn generate_preview(content: &str, preview_size: usize) -> String {
    if content.len() <= preview_size {
        return content.to_string();
    }
    
    let chars: Vec<char> = content.chars().take(preview_size).collect();
    format!("{}... [truncated]", chars.iter().collect::<String>())
}

/// 构建大工具结果消息
pub fn build_large_tool_result_message(
    tool_name: &str,
    preview: &str,
    file_path: &str,
    total_size: usize,
) -> String {
    format!(
        "[{}] Output truncated ({} bytes). Full output saved to: {}\n\nPreview:\n{}",
        tool_name, total_size, file_path, preview
    )
}

/// 持久化工具结果到文件
pub fn persist_tool_result(
    tool_results_dir: &str,
    tool_use_id: &str,
    content: &str,
) -> std::io::Result<(String, usize)> {
    let file_path = get_tool_result_path(tool_results_dir, tool_use_id);
    let size = content.len();
    std::fs::write(&file_path, content)?;
    Ok((file_path, size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_builder() {
        let tool = ToolBuilder::new("test", "A test tool")
            .input_schema(json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                }
            }))
            .read_only()
            .concurrency_safe()
            .interrupt_behavior(InterruptBehavior::Cancel)
            .execute_simple(|input: serde_json::Value, _ctx: &ToolContext| async move {
                Ok(input)
            })
            .build();

        assert_eq!(tool.name(), "test");
        assert!(tool.is_read_only(&serde_json::Value::Null));
        assert!(tool.is_concurrency_safe(&serde_json::Value::Null));
        assert_eq!(tool.interrupt_behavior(), InterruptBehavior::Cancel);
    }

    #[test]
    fn test_tool_builder_fail_closed_defaults() {
        let tool = ToolBuilder::<serde_json::Value, serde_json::Value>::new("test", "desc")
            .input_schema(json!({}))
            .execute_simple(|_, _| async { Ok(serde_json::Value::Null) })
            .build();

        // 验证 fail-closed 默认值
        assert!(!tool.is_concurrency_safe(&serde_json::Value::Null));
        assert!(!tool.is_read_only(&serde_json::Value::Null));
        assert!(!tool.is_destructive(&serde_json::Value::Null));
        assert_eq!(tool.permission_level(), ToolPermissionLevel::RequiresConfirmation);
    }

    #[test]
    fn test_tool_result_persist() {
        let temp_dir = std::env::temp_dir().to_string_lossy().to_string();
        let tool_results_dir = ensure_tool_results_dir(&temp_dir).unwrap();
        
        let content = "Hello, World!";
        let (file_path, size) = persist_tool_result(&tool_results_dir, "test-1", content).unwrap();
        
        assert!(file_path.contains("test-1.json"));
        assert_eq!(size, content.len());
        
        // 清理
        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_generate_preview() {
        let content = "Hello, World!";
        let preview = generate_preview(content, 100);
        assert_eq!(preview, content);
        
        let content = "A".repeat(200);
        let preview = generate_preview(&content, 50);
        assert!(preview.ends_with("... [truncated]"));
        assert!(preview.len() < content.len());
    }

    #[test]
    fn test_build_large_tool_result_message() {
        let message = build_large_tool_result_message(
            "read",
            "Preview content",
            "/path/to/file.json",
            10000,
        );
        
        assert!(message.contains("[read]"));
        assert!(message.contains("10000 bytes"));
        assert!(message.contains("/path/to/file.json"));
        assert!(message.contains("Preview content"));
    }
}
