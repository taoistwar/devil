//! MCP 工具集成模块
//!
//! 将 MCP 服务器发现的工具适配到 Agent 工具系统

use crate::tools::tool::Tool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP 工具适配器
///
/// 将 MCP 服务器发现的工具适配为 Agent 的 Tool trait 实现
pub struct McpToolAdapter {
    /// 全局唯一名称 (mcp__{server}__{tool})
    name: String,
    /// 原始工具名
    original_name: String,
    /// 服务器标识符
    server_id: String,
    /// 工具描述
    description: String,
    /// 输入 Schema
    input_schema: serde_json::Value,
    /// 是否已授权
    is_authorized: bool,
}

/// MCP 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInput {
    /// 工具参数 (JSON)
    pub arguments: serde_json::Value,
}

/// MCP 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolOutput {
    /// 工具执行结果
    pub content: Vec<McpContentBlock>,
    /// 是否有错误
    pub is_error: Option<bool>,
}

/// MCP 内容块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContentBlock {
    /// 文本内容
    #[serde(rename = "text")]
    Text { text: String },
    /// 图片内容
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    /// 资源内容
    #[serde(rename = "resource")]
    Resource {
        uri: String,
        mime_type: String,
        text: String,
    },
}

impl McpToolAdapter {
    /// 创建新的 MCP 工具适配器
    pub fn new(
        global_name: String,
        original_name: String,
        server_id: String,
        description: String,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: global_name,
            original_name,
            server_id,
            description,
            input_schema,
            is_authorized: false,
        }
    }

    /// 创建授权的 MCP 工具适配器
    pub fn authorized(
        global_name: String,
        original_name: String,
        server_id: String,
        description: String,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: global_name,
            original_name,
            server_id,
            description,
            input_schema,
            is_authorized: true,
        }
    }

    /// 获取服务器 ID
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// 获取原始工具名
    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    /// 检查是否已授权
    pub fn is_authorized(&self) -> bool {
        self.is_authorized
    }

    /// 设置授权状态
    pub fn set_authorized(&mut self, authorized: bool) {
        self.is_authorized = authorized;
    }
}

/// MCP 工具执行器
///
/// 负责实际调用 MCP 服务器执行工具
pub struct McpToolExecutor {
    /// 工具执行函数
    execute_fn: Box<
        dyn Fn(
                String,
                String,
                serde_json::Value,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<McpToolOutput>> + Send + Sync>,
            > + Send
            + Sync,
    >,
}

impl McpToolExecutor {
    /// 创建新的执行器
    pub fn new(
        execute_fn: impl Fn(
                String,
                String,
                serde_json::Value,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<McpToolOutput>> + Send + Sync>,
            > + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            execute_fn: Box::new(execute_fn),
        }
    }

    /// 执行 MCP 工具
    pub async fn execute(
        &self,
        server_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    ) -> Result<McpToolOutput> {
        (self.execute_fn)(server_id, tool_name, arguments).await
    }
}

/// MCP 工具注册表
///
/// 管理所有 MCP 服务器及其工具
pub struct McpToolRegistry {
    /// 服务器 ID -> MCP 服务器信息
    servers: Arc<RwLock<HashMap<String, McpServerInfo>>>,
    /// 全局工具名 -> MCP 工具信息
    tools: Arc<RwLock<HashMap<String, McpToolInfo>>>,
    /// MCP 工具执行器
    executor: Arc<RwLock<Option<McpToolExecutor>>>,
}

/// MCP 服务器信息
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    /// 服务器 ID
    pub id: String,
    /// 服务器名称
    pub name: String,
    /// 服务器配置
    pub config: serde_json::Value,
    /// 是否已连接
    pub connected: bool,
}

/// MCP 工具信息
#[derive(Debug, Clone)]
pub struct McpToolInfo {
    /// 全局唯一工具名
    pub global_name: String,
    /// 原始工具名
    pub original_name: String,
    /// 服务器 ID
    pub server_id: String,
    /// 工具描述
    pub description: String,
    /// 输入 Schema
    pub input_schema: serde_json::Value,
    /// 是否已授权
    pub is_authorized: bool,
}

impl McpToolRegistry {
    /// 创建新的 MCP 工具注册表
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            executor: Arc::new(RwLock::new(None)),
        }
    }

    /// 设置执行器
    pub fn set_executor(&self, executor: McpToolExecutor) {
        let _executor = Arc::new(RwLock::new(Some(executor)));
    }

    /// 注册 MCP 服务器
    pub async fn register_server(&self, server: McpServerInfo) {
        let mut servers = self.servers.write().await;
        servers.insert(server.id.clone(), server);
    }

    /// 注册 MCP 工具
    pub async fn register_tool(&self, tool: McpToolInfo) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.global_name.clone(), tool);
    }

    /// 获取工具信息
    pub async fn get_tool(&self, global_name: &str) -> Option<McpToolInfo> {
        let tools = self.tools.read().await;
        tools.get(global_name).cloned()
    }

    /// 列出所有已授权的工具
    pub async fn list_authorized_tools(&self) -> Vec<McpToolInfo> {
        let tools = self.tools.read().await;
        tools
            .values()
            .filter(|t| t.is_authorized)
            .cloned()
            .collect()
    }

    /// 列出所有服务器
    pub async fn list_servers(&self) -> Vec<McpServerInfo> {
        let servers = self.servers.read().await;
        servers.values().cloned().collect()
    }
}

impl Default for McpToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 MCP 工具列表创建适配器
pub fn create_tool_adapters(tools: Vec<McpToolInfo>) -> HashMap<String, McpToolAdapter> {
    tools
        .into_iter()
        .map(|tool| {
            let adapter = if tool.is_authorized {
                McpToolAdapter::authorized(
                    tool.global_name.clone(),
                    tool.original_name.clone(),
                    tool.server_id.clone(),
                    tool.description.clone(),
                    tool.input_schema.clone(),
                )
            } else {
                McpToolAdapter::new(
                    tool.global_name.clone(),
                    tool.original_name.clone(),
                    tool.server_id.clone(),
                    tool.description.clone(),
                    tool.input_schema.clone(),
                )
            };
            (tool.global_name, adapter)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_adapter_creation() {
        let adapter = McpToolAdapter::new(
            "mcp__filesystem__read".to_string(),
            "read".to_string(),
            "filesystem".to_string(),
            "Read a file from the filesystem".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        );

        assert_eq!(adapter.name, "mcp__filesystem__read");
        assert_eq!(adapter.original_name, "read");
        assert_eq!(adapter.server_id, "filesystem");
        assert!(!adapter.is_authorized());
    }

    #[tokio::test]
    async fn test_mcp_tool_registry() {
        let registry = McpToolRegistry::new();

        // 注册服务器
        registry
            .register_server(McpServerInfo {
                id: "filesystem".to_string(),
                name: "Filesystem Server".to_string(),
                config: serde_json::json!({}),
                connected: true,
            })
            .await;

        // 注册工具
        registry
            .register_tool(McpToolInfo {
                global_name: "mcp__filesystem__read".to_string(),
                original_name: "read".to_string(),
                server_id: "filesystem".to_string(),
                description: "Read a file".to_string(),
                input_schema: serde_json::json!({}),
                is_authorized: true,
            })
            .await;

        // 验证
        let tools = registry.list_authorized_tools().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].global_name, "mcp__filesystem__read");
    }
}
