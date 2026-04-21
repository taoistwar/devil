//! MCP 工具发现与映射
//!
//! 负责：
//! - 从 MCP 服务器发现工具（tools/list）
//! - 工具名称映射（添加 mcp__{server}__{tool} 前缀）
//! - 工具元数据缓存
//! - Unicode 字符清理

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 工具定义（来自 MCP tools/list 响应）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpTool {
    /// 工具名称（来自服务器）
    pub name: String,
    /// 工具描述
    pub description: Option<String>,
    /// 输入 Schema（JSON Schema）
    pub input_schema: serde_json::Value,
}

/// 映射后的工具条目
#[derive(Debug, Clone)]
pub struct MappedTool {
    /// 全局唯一工具名（mcp__{server}__{tool}）
    pub global_name: String,
    /// 原始工具名
    pub original_name: String,
    /// 服务器标识符
    pub server_id: String,
    /// 工具描述
    pub description: String,
    /// 输入 Schema
    pub input_schema: serde_json::Value,
    /// 是否已授权
    pub is_authorized: bool,
}

/// 工具发现器
pub struct ToolDiscoverer {
    /// 服务器 ID -> 工具列表
    tools: Arc<RwLock<HashMap<String, Vec<MappedTool>>>>,
    /// 全局工具名 -> 服务器 ID + 原始名
    tool_index: Arc<RwLock<HashMap<String, (String, String)>>>,
}

impl ToolDiscoverer {
    /// 创建新的工具发现器
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            tool_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 从服务器发现工具
    ///
    /// # Arguments
    ///
    /// * `server_id` - 服务器标识符
    /// * `raw_tools` - 从服务器获取的原始工具列表
    /// * `name_prefix_strategy` - 名称前缀策略（ServerName/ServerId/None/Custom）
    pub async fn discover_tools(
        &self,
        server_id: &str,
        raw_tools: Vec<McpTool>,
        name_prefix_strategy: &str,
    ) -> Result<Vec<MappedTool>> {
        info!(
            "Discovering {} tools from server: {}",
            raw_tools.len(),
            server_id
        );

        let mut mapped_tools = Vec::with_capacity(raw_tools.len());

        for tool in raw_tools {
            // 清理工具名称（移除 Unicode 控制字符）
            let clean_name = clean_unicode(&tool.name);

            if clean_name.is_empty() {
                warn!("Tool name is empty after cleaning, skipping");
                continue;
            }

            // 构建全局唯一名称
            let global_name = match name_prefix_strategy {
                "ServerName" | "ServerId" => {
                    format!("mcp__{}__{}", server_id, clean_name)
                }
                "None" => clean_name.clone(),
                custom => {
                    format!("mcp__{}__{}", custom, clean_name)
                }
            };

            debug!("Mapping tool: {} -> {}", clean_name, global_name);

            let mapped = MappedTool {
                global_name: global_name.clone(),
                original_name: clean_name,
                server_id: server_id.to_string(),
                description: tool.description.unwrap_or_default(),
                input_schema: tool.input_schema,
                is_authorized: false, // 初始为未授权，等待权限检查
            };

            mapped_tools.push(mapped);
        }

        // 更新缓存
        {
            let mut tools_map = self.tools.write().await;
            tools_map.insert(server_id.to_string(), mapped_tools.clone());
        }

        {
            let mut index = self.tool_index.write().await;
            for tool in &mapped_tools {
                index.insert(
                    tool.global_name.clone(),
                    (server_id.to_string(), tool.original_name.clone()),
                );
            }
        }

        Ok(mapped_tools)
    }

    /// 获取服务器的所有工具
    pub async fn get_tools(&self, server_id: &str) -> Vec<MappedTool> {
        let tools_map = self.tools.read().await;
        tools_map.get(server_id).cloned().unwrap_or_default()
    }

    /// 根据全局工具名查找工具
    pub async fn find_tool(&self, global_name: &str) -> Option<MappedTool> {
        let index = self.tool_index.read().await;

        let (server_id, original_name) = index.get(global_name)?;
        let tools_map = self.tools.read().await;
        let tools = tools_map.get(server_id)?;
        tools
            .iter()
            .find(|t| t.original_name == *original_name)
            .cloned()
    }

    /// 更新工具授权状态
    pub async fn update_authorization(&self, global_name: &str, is_authorized: bool) -> Result<()> {
        let mut tools_map = self.tools.write().await;

        for tools in tools_map.values_mut() {
            if let Some(tool) = tools.iter_mut().find(|t| t.global_name == global_name) {
                tool.is_authorized = is_authorized;
                return Ok(());
            }
        }

        anyhow::bail!("Tool not found: {}", global_name)
    }

    /// 清除服务器的工具缓存
    pub async fn clear_server_tools(&self, server_id: &str) {
        let mut tools_map = self.tools.write().await;
        let mut index = self.tool_index.write().await;

        if let Some(tools) = tools_map.remove(server_id) {
            for tool in tools {
                index.remove(&tool.global_name);
            }
            info!("Cleared tools cache for server: {}", server_id);
        }
    }

    /// 获取所有已授权的工具
    pub async fn get_authorized_tools(&self) -> Vec<MappedTool> {
        let tools_map = self.tools.read().await;
        tools_map
            .values()
            .flatten()
            .filter(|t| t.is_authorized)
            .cloned()
            .collect()
    }

    /// 获取工具总数统计
    pub async fn get_stats(&self) -> ToolStats {
        let tools_map = self.tools.read().await;

        let total: usize = tools_map.values().map(|v| v.len()).sum();
        let authorized: usize = tools_map
            .values()
            .flatten()
            .filter(|t| t.is_authorized)
            .count();

        ToolStats {
            total,
            authorized,
            unauthorized: total - authorized,
            servers: tools_map.len(),
        }
    }
}

/// 工具统计信息
#[derive(Debug, Clone)]
pub struct ToolStats {
    pub total: usize,
    pub authorized: usize,
    pub unauthorized: usize,
    pub servers: usize,
}

/// 清理 Unicode 控制字符和其他非法字符
pub fn clean_unicode(name: &str) -> String {
    name.chars()
        .filter(|c| {
            // 移除 Unicode 控制字符（U+0000 到 U+001F，以及 U+007F 到 U+009F）
            let code = *c as u32;
            !(code <= 0x1F || (code >= 0x7F && code <= 0x9F))
        })
        .collect()
}

/// 解析工具名称前缀策略
pub fn parse_name_prefix_strategy(
    strategy: &str,
    server_name: Option<&str>,
    server_id: &str,
) -> String {
    match strategy {
        "ServerName" => server_name.unwrap_or(server_id).to_string(),
        "ServerId" => server_id.to_string(),
        "None" => String::new(),
        custom => custom.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_unicode() {
        // 测试移除控制字符
        assert_eq!(clean_unicode("hello\u{0000}world"), "helloworld");
        assert_eq!(clean_unicode("test\u{001F}data"), "testdata");
        assert_eq!(clean_unicode("foo\u{007F}bar"), "foobar");

        // 保留正常字符
        assert_eq!(clean_unicode("hello-world"), "hello-world");
        assert_eq!(clean_unicode("工具_123"), "工具_123");
    }

    #[test]
    fn test_tool_name_mapping() {
        let global = format_mcp_tool_name("myserver", "bash");
        assert_eq!(global, "mcp__myserver__bash");

        let global = format_mcp_tool_name("git", "commit");
        assert_eq!(global, "mcp__git__commit");
    }
}

/// 格式化 MCP 工具名为全局唯一名称
pub fn format_mcp_tool_name(server_id: &str, tool_name: &str) -> String {
    format!("mcp__{}__{}", server_id, tool_name)
}

/// 从全局工具名解析出服务器 ID 和原始工具名
pub fn parse_mcp_tool_name(global_name: &str) -> Option<(&str, &str)> {
    if !global_name.starts_with("mcp__") {
        return None;
    }

    let parts: Vec<&str> = global_name[5..].split("__").collect();
    if parts.len() != 2 {
        return None;
    }

    Some((parts[0], parts[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_unicode() {
        // 测试移除控制字符
        assert_eq!(clean_unicode("hello\u{0000}world"), "helloworld");
        assert_eq!(clean_unicode("test\u{001F}data"), "testdata");
        assert_eq!(clean_unicode("foo\u{007F}bar"), "foobar");

        // 保留正常字符
        assert_eq!(clean_unicode("hello-world"), "hello-world");
        assert_eq!(clean_unicode("工具_123"), "工具_123");
    }

    #[test]
    fn test_tool_name_mapping() {
        let global = format_mcp_tool_name("myserver", "bash");
        assert_eq!(global, "mcp__myserver__bash");

        let global = format_mcp_tool_name("git", "commit");
        assert_eq!(global, "mcp__git__commit");
    }

    #[test]
    fn test_tool_name_parsing() {
        let (server, tool) = parse_mcp_tool_name("mcp__filesystem__read_file").unwrap();
        assert_eq!(server, "filesystem");
        assert_eq!(tool, "read_file");

        // 无效格式
        assert!(parse_mcp_tool_name("invalid").is_none());
        assert!(parse_mcp_tool_name("mcp__only").is_none());
        assert!(parse_mcp_tool_name("mcp__a__b__c").is_none());
    }

    #[test]
    fn test_parse_name_prefix_strategy() {
        assert_eq!(
            parse_name_prefix_strategy("ServerName", Some("MyServer"), "id123"),
            "MyServer"
        );
        assert_eq!(
            parse_name_prefix_strategy("ServerName", None, "id123"),
            "id123"
        );
        assert_eq!(
            parse_name_prefix_strategy("ServerId", Some("MyServer"), "id123"),
            "id123"
        );
        assert_eq!(
            parse_name_prefix_strategy("None", Some("MyServer"), "id123"),
            ""
        );
        assert_eq!(
            parse_name_prefix_strategy("custom", Some("MyServer"), "id123"),
            "custom"
        );
    }

    #[tokio::test]
    async fn test_tool_discoverer() {
        let discoverer = ToolDiscoverer::new();

        let raw_tools = vec![
            McpTool {
                name: "bash".to_string(),
                description: Some("Execute bash commands".to_string()),
                input_schema: serde_json::json!({"type": "object"}),
            },
            McpTool {
                name: "read_file".to_string(),
                description: None,
                input_schema: serde_json::json!({"type": "object"}),
            },
        ];

        let mapped = discoverer
            .discover_tools("test-server", raw_tools, "ServerId")
            .await
            .unwrap();

        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0].global_name, "mcp__test-server__bash");
        assert_eq!(mapped[1].global_name, "mcp__test-server__read_file");
        assert_eq!(mapped[0].description, "Execute bash commands");
        assert_eq!(mapped[1].description, "");

        // 测试工具查找
        let tool = discoverer.find_tool("mcp__test-server__bash").await;
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().original_name, "bash");

        // 测试统计
        let stats = discoverer.get_stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.authorized, 0);
        assert_eq!(stats.unauthorized, 2);
        assert_eq!(stats.servers, 1);
    }

    #[tokio::test]
    async fn test_authorization_update() {
        let discoverer = ToolDiscoverer::new();

        let raw_tools = vec![McpTool {
            name: "safe_tool".to_string(),
            description: None,
            input_schema: serde_json::json!({"type": "object"}),
        }];

        discoverer
            .discover_tools("server1", raw_tools, "ServerId")
            .await
            .unwrap();

        // 初始未授权
        let tool = discoverer
            .find_tool("mcp__server1__safe_tool")
            .await
            .unwrap();
        assert!(!tool.is_authorized);

        // 更新为已授权
        discoverer
            .update_authorization("mcp__server1__safe_tool", true)
            .await
            .unwrap();

        let tool = discoverer
            .find_tool("mcp__server1__safe_tool")
            .await
            .unwrap();
        assert!(tool.is_authorized);

        // 测试获取已授权工具
        let authorized = discoverer.get_authorized_tools().await;
        assert_eq!(authorized.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_server_tools() {
        let discoverer = ToolDiscoverer::new();

        let raw_tools = vec![McpTool {
            name: "tool1".to_string(),
            description: None,
            input_schema: serde_json::json!({"type": "object"}),
        }];

        discoverer
            .discover_tools("server1", raw_tools.clone(), "ServerId")
            .await
            .unwrap();
        discoverer
            .discover_tools("server2", raw_tools, "ServerId")
            .await
            .unwrap();

        let stats = discoverer.get_stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.servers, 2);

        // 清除 server1 的工具
        discoverer.clear_server_tools("server1").await;

        let stats = discoverer.get_stats().await;
        assert_eq!(stats.total, 1);
        assert_eq!(stats.servers, 1);

        // server1 的工具应该找不到
        let tool = discoverer.find_tool("mcp__server1__tool1").await;
        assert!(tool.is_none());
    }
}
