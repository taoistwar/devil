//! 工具系统模块
//! 
//! 定义工具接口和执行上下文，包括：
//! - Tool trait: 工具接口定义
//! - ToolContext: 工具执行上下文
//! - ToolResult: 工具执行结果
//! - ToolUseBlock: 工具调用块

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工具使用块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// 工具调用 ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 输入参数
    pub input: serde_json::Value,
}

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
pub struct ToolResult {
    /// 工具调用 ID
    pub tool_use_id: String,
    /// 是否执行成功
    pub is_success: bool,
    /// 结果内容
    pub content: String,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

impl ToolResult {
    /// 创建成功的结果
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            is_success: true,
            content: content.into(),
            error: None,
        }
    }

    /// 创建失败的结果
    pub fn error(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            is_success: false,
            content: content.into(),
            error: Some(error.into()),
        }
    }
}

/// 工具权限级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermissionLevel {
    /// 只读操作，无需确认
    ReadOnly,
    /// 可能产生副作用，需要确认
    RequiresConfirmation,
    /// 破坏性操作，严格受限
    Destructive,
}

/// 工具元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入 JSON Schema
    pub input_schema: serde_json::Value,
    /// 权限级别
    pub permission_level: ToolPermissionLevel,
    /// 是否支持并发执行
    pub concurrency_safe: bool,
    /// 超时时间（秒）
    pub timeout_secs: Option<u64>,
}

/// 工具接口 Trait
/// 
/// 每个工具必须实现此接口
#[async_trait]
pub trait Tool: Send + Sync {
    /// 获取工具名称
    fn name(&self) -> &str;

    /// 获取工具描述
    fn description(&self) -> &str;

    /// 获取输入 JSON Schema
    fn input_schema(&self) -> serde_json::Value;

    /// 判断权限级别
    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    /// 判断是否支持并发执行
    fn concurrency_safe(&self) -> bool {
        false
    }

    /// 执行工具
    async fn execute(&self, input: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult>;
}

/// 工具注册表
/// 
/// 管理所有已注册的工具
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// 创建空的工具注册表
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> Result<()> {
        let name = tool.name().to_string();
        if self.tools.contains_key(&name) {
            anyhow::bail!("Tool already registered: {}", name);
        }
        self.tools.insert(name, Box::new(tool));
        Ok(())
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }

    /// 列出所有工具
    pub fn list_tools(&self) -> Vec<ToolMetadata> {
        self.tools
            .values()
            .map(|tool| ToolMetadata {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
                permission_level: tool.permission_level(),
                concurrency_safe: tool.concurrency_safe(),
                timeout_secs: None,
            })
            .collect()
    }

    /// 获取工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具执行器
/// 
/// 负责根据并发策略执行工具调用
pub struct ToolExecutor {
    /// 并发执行时的最大并行度
    max_concurrency: usize,
}

impl ToolExecutor {
    /// 创建新的工具执行器
    pub fn new(max_concurrency: usize) -> Self {
        Self { max_concurrency }
    }

    /// 执行单个工具
    pub async fn execute_tool(
        &self,
        tool: &dyn Tool,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        tool.execute(input, ctx).await
    }

    /// 批量执行工具（支持并发控制）
    pub async fn execute_batch(
        &self,
        tools: Vec<(&dyn Tool, serde_json::Value)>,
        ctx: &ToolContext,
    ) -> Vec<Result<ToolResult>> {
        // 简单的串行实现
        // TODO: 实现并发分区调度
        let mut results = Vec::new();
        for (tool, input) in tools {
            let result = tool.execute(input, ctx).await;
            results.push(result);
        }
        results
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoTool;

    #[async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }

        fn description(&self) -> &str {
            "Echo back the input"
        }

        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo back"
                    }
                },
                "required": ["message"]
            })
        }

        async fn execute(
            &self,
            input: serde_json::Value,
            _ctx: &ToolContext,
        ) -> Result<ToolResult> {
            let message = input["message"].as_str().unwrap_or("");
            Ok(ToolResult::success("echo-1", format!("Echo: {}", message)))
        }
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        assert!(registry.is_empty());

        registry.register(EchoTool).unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.get("echo").is_some());
        assert!(registry.get("nonexistent").is_none());

        // 不允许重复注册
        assert!(registry.register(EchoTool).is_err());
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let executor = ToolExecutor::new(4);
        let tool = EchoTool;
        let ctx = ToolContext::default();

        let input = serde_json::json!({ "message": "Hello" });
        let result = executor.execute_tool(&tool, input, &ctx).await.unwrap();

        assert!(result.is_success);
        assert!(result.content.contains("Echo: Hello"));
        assert_eq!(result.error, None);
    }

    #[test]
    fn test_tool_result_creation() {
        let success = ToolResult::success("id-1", "Success content");
        assert!(success.is_success);
        assert!(success.error.is_none());

        let error = ToolResult::error("id-2", "Error content", "Something went wrong");
        assert!(!error.is_success);
        assert!(error.error.is_some());
    }
}
