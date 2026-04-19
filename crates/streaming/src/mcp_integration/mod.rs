//! MCP 与 Streaming 集成模块
//!
//! 实现：
//! - QueryEngine 与 MCP 服务器集成
//! - StreamingToolExecutor 调用 MCP 工具
//! - ForkedAgent 使用 MCP 缓存共享

use anyhow::{Context, Result};
use futures::future::FutureExt;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::query_engine::{Message, QueryDeps, QueryEngine, StreamEvent};
use crate::streaming_tool_executor::{StreamingToolExecutor, ToolResult, TrackedTool};
use devil_mcp::{
    MappedTool, McpConnectionManager, PermissionChecker, ToolDiscoverer,
};

/// MCP-Aware QueryDeps 实现
pub struct McpQueryDeps {
    /// MCP 连接管理器
    mcp_manager: Arc<McpConnectionManager>,
    /// 权限检查器
    permission_checker: Arc<PermissionChecker>,
    /// 工具发现器
    tool_discoverer: Arc<ToolDiscoverer>,
    /// MCP 工具名称 -> 全局名称映射
    tool_name_map: Arc<RwLock<HashMap<String, String>>>,
}

impl McpQueryDeps {
    /// 创建新的 MCP QueryDeps
    pub fn new(
        mcp_manager: Arc<McpConnectionManager>,
        permission_checker: Arc<PermissionChecker>,
        tool_discoverer: Arc<ToolDiscoverer>,
    ) -> Self {
        Self {
            mcp_manager,
            permission_checker,
            tool_discoverer,
            tool_name_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 初始化 MCP 工具发现
    pub async fn initialize_mcp_tools(&self) -> Result<Vec<MappedTool>> {
        info!("Initializing MCP tools");

        let mut all_tools = Vec::new();

        // 从所有已连接的 MCP 服务器发现工具
        let servers = self.mcp_manager.list_servers().await;

        for server_id in servers {
            // 检查服务器权限
            match self.permission_checker.check_server(&server_id).await {
                devil_mcp::PermissionResult::Allowed => {
                    info!("Server {} is allowed", server_id);
                }
                devil_mcp::PermissionResult::Denied(reason) => {
                    warn!("Server {} denied: {}", server_id, reason);
                    continue;
                }
                devil_mcp::PermissionResult::NeedsConfirmation => {
                    warn!("Server {} needs confirmation", server_id);
                    continue;
                }
            }

            // 从服务器发现工具
            let tools = self.tool_discoverer.get_tools(&server_id).await;

            for tool in tools {
                let global_name = tool.global_name.clone();
                let original_name = tool.original_name.clone();

                // 建立名称映射
                self.tool_name_map
                    .write()
                    .await
                    .insert(original_name.clone(), global_name.clone());

                // 检查工具权限
                match self.permission_checker.check_tool(&global_name).await {
                    devil_mcp::PermissionResult::Allowed => {
                        self.tool_discoverer
                            .update_authorization(&global_name, true)
                            .await?;
                        all_tools.push(tool);
                        debug!("Tool {} authorized", global_name);
                    }
                    _ => {
                        debug!("Tool {} not authorized", global_name);
                    }
                }
            }
        }

        info!("Discovered {} MCP tools", all_tools.len());
        Ok(all_tools)
    }

    /// 执行 MCP 工具
    pub async fn execute_mcp_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String> {
        // 查找全局工具名
        let tool_map = self.tool_name_map.read().await;
        let global_name = tool_map
            .get(tool_name)
            .with_context(|| format!("Tool not found: {}", tool_name))?;

        // 检查工具权限
        match self.permission_checker.check_tool(global_name).await {
            devil_mcp::PermissionResult::Allowed => {}
            devil_mcp::PermissionResult::Denied(reason) => {
                anyhow::bail!("Tool {} denied: {}", global_name, reason);
            }
            devil_mcp::PermissionResult::NeedsConfirmation => {
                anyhow::bail!("Tool {} needs confirmation", global_name);
            }
        }

        // 解析工具名获取服务器 ID
        let parts: Vec<&str> = global_name.split("__").collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid tool name format: {}", global_name);
        }
        let server_id = parts[1];
        let original_tool_name = parts[2];

        // 通过 MCP Bridge 调用工具
        info!(
            "Executing MCP tool: {} on server {}",
            original_tool_name, server_id
        );

        // TODO: 实际调用 MCP Bridge
        // let bridge = self.mcp_manager.get_bridge(server_id).await?;
        // let result = bridge.call_tool(original_tool_name, arguments).await?;

        // 模拟结果
        let result = serde_json::json!({
            "status": "success",
            "tool": original_tool_name,
            "server": server_id,
            "arguments": arguments,
        });

        Ok(result.to_string())
    }
}

impl QueryDeps for McpQueryDeps {
    fn call_model(
        &self,
        _messages: &[Message],
        _stream: bool,
    ) -> futures::stream::BoxStream<'static, Result<StreamEvent>> {
        // TODO: 实际调用 LLM API
        // 这里返回模拟流
        use futures::stream;
        stream::empty().boxed()
    }

    fn execute_tool(
        &self,
        tool: &TrackedTool,
    ) -> futures::future::BoxFuture<'static, Result<String>> {
        let this = self.clone();
        let tool_name = tool.name.clone();
        let arguments = tool.input.clone();

        async move { this.execute_mcp_tool(&tool_name, arguments).await }.boxed()
    }
}

impl Clone for McpQueryDeps {
    fn clone(&self) -> Self {
        Self {
            mcp_manager: self.mcp_manager.clone(),
            permission_checker: self.permission_checker.clone(),
            tool_discoverer: self.tool_discoverer.clone(),
            tool_name_map: self.tool_name_map.clone(),
        }
    }
}

/// 创建集成 MCP 的 QueryEngine
pub fn create_mcp_query_engine(
    mcp_manager: Arc<McpConnectionManager>,
    permission_checker: Arc<PermissionChecker>,
    tool_discoverer: Arc<ToolDiscoverer>,
) -> QueryEngine<McpQueryDeps> {
    let deps = McpQueryDeps::new(mcp_manager, permission_checker, tool_discoverer);
    QueryEngine::new(deps)
}

/// MCP 工具调用转换器
pub struct McpToolConverter;

impl McpToolConverter {
    /// 将 MCP 工具转换为 TrackedTool
    pub fn to_tracked_tool(mapped_tool: &MappedTool) -> TrackedTool {
        let is_concurrency_safe = Self::is_tool_safe(&mapped_tool.original_name);

        TrackedTool::new(
            mapped_tool.global_name.clone(),
            mapped_tool.original_name.clone(),
            serde_json::Value::Null, // 实际调用时填充
            is_concurrency_safe,
        )
    }

    /// 判断工具是否并发安全
    fn is_tool_safe(tool_name: &str) -> bool {
        let name = tool_name.to_lowercase();

        // 只读工具
        let safe = [
            "read", "grep", "glob", "fetch", "list", "search", "get", "query", "select", "show",
        ];

        // 写入工具
        let unsafe_ = [
            "write", "edit", "delete", "remove", "create", "bash", "shell", "exec", "run", "update",
        ];

        if unsafe_.iter().any(|&t| name.contains(t)) {
            return false;
        }

        if safe.iter().any(|&t| name.contains(t)) {
            return true;
        }

        false
    }

    /// 将工具结果转换为 MCP 格式
    pub fn result_to_mcp_format(result: &ToolResult) -> serde_json::Value {
        serde_json::json!({
            "tool_use_id": result.tool_use_id,
            "content": result.content,
            "is_error": result.is_error,
        })
    }
}

/// 构建集成环境
pub struct McpStreamingIntegration {
    /// MCP 连接管理器
    pub mcp_manager: Arc<McpConnectionManager>,
    /// 权限检查器
    pub permission_checker: Arc<PermissionChecker>,
    /// 工具发现器
    pub tool_discoverer: Arc<ToolDiscoverer>,
    /// QueryEngine
    pub query_engine: Option<QueryEngine<McpQueryDeps>>,
    /// 工具执行器
    pub tool_executor: StreamingToolExecutor,
}

impl McpStreamingIntegration {
    /// 创建新的集成环境
    pub fn new(
        mcp_manager: Arc<McpConnectionManager>,
        permission_checker: Arc<PermissionChecker>,
        tool_discoverer: Arc<ToolDiscoverer>,
    ) -> Self {
        Self {
            mcp_manager,
            permission_checker,
            tool_discoverer,
            query_engine: None,
            tool_executor: StreamingToolExecutor::new(),
        }
    }

    /// 初始化
    pub async fn initialize(&mut self) -> Result<Vec<MappedTool>> {
        info!("Initializing MCP-Streaming integration");

        // 创建 QueryEngine
        let deps = McpQueryDeps::new(
            self.mcp_manager.clone(),
            self.permission_checker.clone(),
            self.tool_discoverer.clone(),
        );
        self.query_engine = Some(QueryEngine::new(deps));

        // 初始化 MCP 工具
        let tools = self.discover_tools().await?;

        info!(
            "MCP-Streaming integration initialized with {} tools",
            tools.len()
        );

        Ok(tools)
    }

    /// 发现并注册工具
    pub async fn discover_tools(&self) -> Result<Vec<MappedTool>> {
        let deps = McpQueryDeps::new(
            self.mcp_manager.clone(),
            self.permission_checker.clone(),
            self.tool_discoverer.clone(),
        );

        deps.initialize_mcp_tools().await
    }

    /// 获取 QueryEngine
    pub fn get_query_engine(&self) -> Option<&QueryEngine<McpQueryDeps>> {
        self.query_engine.as_ref()
    }

    /// 获取工具执行器
    pub fn get_tool_executor(&self) -> &StreamingToolExecutor {
        &self.tool_executor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_tool_safety_detection() {
        assert!(McpToolConverter::is_tool_safe("read_file"));
        assert!(McpToolConverter::is_tool_safe("grep"));
        assert!(McpToolConverter::is_tool_safe("glob"));

        assert!(!McpToolConverter::is_tool_safe("bash"));
        assert!(!McpToolConverter::is_tool_safe("write_file"));
        assert!(!McpToolConverter::is_tool_safe("edit"));
    }

    #[test]
    fn test_result_to_mcp_format() {
        let result = ToolResult {
            tool_use_id: "tool-1".to_string(),
            content: "test content".to_string(),
            is_error: false,
        };

        let json = McpToolConverter::result_to_mcp_format(&result);

        assert_eq!(json["tool_use_id"], "tool-1");
        assert_eq!(json["content"], "test content");
        assert_eq!(json["is_error"], false);
    }

    #[tokio::test]
    async fn test_integration_creation() {
        let mcp_manager = Arc::new(McpConnectionManager::new());
        let permission_checker = Arc::new(PermissionChecker::new(
            devil_mcp::EnterprisePolicy::default(),
            devil_mcp::IdeWhitelist::default(),
            devil_mcp::UserPermissions::default(),
        ));
        let tool_discoverer = Arc::new(ToolDiscoverer::new());

        let mut integration =
            McpStreamingIntegration::new(mcp_manager, permission_checker, tool_discoverer);

        // 初始化（不会有实际服务器连接）
        let tools = integration.initialize().await.unwrap();

        // 初始应该没有工具
        assert_eq!(tools.len(), 0);

        // QueryEngine 应该已创建
        assert!(integration.get_query_engine().is_some());
    }
}
