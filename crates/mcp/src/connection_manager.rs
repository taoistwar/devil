//! MCP 连接管理器
//!
//! 实现 MCP 连接的集中管理，包括：
//! - 连接池（按服务器名索引）
//! - 状态机管理（5 种状态）
//! - 指数退避重试
//! - 7 层配置作用域加载

use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// MCP 连接
pub struct McpConnection {
    /// 服务器配置
    pub config: Arc<McpServerConfig>,
    /// 当前状态
    pub state: McpConnectionState,
}

unsafe impl Sync for McpConnection {}

/// MCP 重新连接结果
pub struct McpReconnectResult {
    /// 工具列表
    pub tools: Vec<MappedTool>,
    /// 命令列表
    pub commands: Vec<String>,
    /// 资源列表
    pub resources: Vec<String>,
}

/// MCP 连接管理器
///
/// 为整个组件树提供 MCP 连接管理能力
pub struct McpConnectionManager {
    /// 连接池（按服务器名称索引）
    connections: RwLock<HashMap<String, McpConnection>>,
    /// 配置来源
    config_sources: ConfigSources,
}

unsafe impl Sync for McpConnectionManager {}

/// 配置来源
pub struct ConfigSources {
    // 实际实现中会包含各作用域的配置加载器
}

impl McpConnectionManager {
    /// 创建连接管理器
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            config_sources: ConfigSources {},
        }
    }

    /// 列出所有已连接的服务器
    pub async fn list_servers(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// 重新连接指定服务器
    ///
    /// # Arguments
    /// * `server_name` - 服务器名称
    ///
    /// # Returns
    /// 更新后的工具列表、命令列表和资源列表
    pub async fn reconnect(&self, server_name: &str) -> Result<McpReconnectResult, anyhow::Error> {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(server_name) {
            info!("Reconnecting to MCP server: {}", server_name);

            // 1. 断开现有连接
            if let McpConnectionState::Connected { cleanup, .. } = &mut conn.state {
                let cleanup_fn = Box::new(cleanup as &mut dyn FnOnce());
                // cleanup_fn(); // 实际调用清理函数
            }

            // 2. 清理工具注册（在外部工具系统中完成）
            // 3. 重新建立连接
            // 4. 发现并注册新工具

            conn.state = McpConnectionState::Connected {
                capabilities: ServerCapabilities {
                    tools: Some(ToolsCapability {
                        list_changed: Some(true),
                    }),
                    prompts: None,
                    resources: None,
                },
                cleanup: Box::new(|| {}),
            };

            Ok(McpReconnectResult {
                tools: vec![],
                commands: vec![],
                resources: vec![],
            })
        } else {
            Err(anyhow::anyhow!("Server not found: {}", server_name))
        }
    }

    /// 启用/禁用服务器
    pub async fn toggle(&self, server_name: &str, enable: bool) -> Result<(), anyhow::Error> {
        if enable {
            self.enable_server(server_name).await
        } else {
            self.disable_server(server_name).await
        }
    }

    /// 从配置加载所有服务器
    pub async fn load_from_config(&self) -> Result<McpLoadResult, anyhow::Error> {
        let mut result = McpLoadResult::default();

        let mut connections = self.connections.write().await;

        // 1. 加载 7 个作用域的配置
        // 2. 去重（插件 vs 手动配置）
        // 3. 应用企业策略过滤
        // 4. 建立连接

        info!("Loaded {} MCP servers", result.allowed.len());

        Ok(result)
    }

    /// 启用服务器
    async fn enable_server(&self, server_name: &str) -> Result<(), anyhow::Error> {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(server_name) {
            info!("Enabling MCP server: {}", server_name);
            // 尝试建立连接
            conn.state = McpConnectionState::Pending {
                retry_count: 0,
                max_retries: 5,
                next_retry_time: std::time::Instant::now(),
            };
        } else {
            warn!("Server {} not found, cannot enable", server_name);
        }

        Ok(())
    }

    /// 禁用服务器
    async fn disable_server(&self, server_name: &str) -> Result<(), anyhow::Error> {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(server_name) {
            info!("Disabling MCP server: {}", server_name);

            // 断开连接
            if let McpConnectionState::Connected { cleanup, .. } = &mut conn.state {
                let _ = std::mem::replace(cleanup, Box::new(|| {}));
            }

            conn.state = McpConnectionState::Disabled;

            // 移除其所有工具（在工具注册表中）
        } else {
            warn!("Server {} not found, cannot disable", server_name);
        }

        Ok(())
    }
}

impl Default for McpConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 企业策略过滤器
pub struct EnterprisePolicyFilter {
    /// 黑名单（deniedMcpServers）
    denied: Vec<ServerMatcher>,
    /// 白名单（allowedMcpServers）
    allowed: Option<Vec<ServerMatcher>>,
}

/// 服务器匹配器（支持名称、命令、URL 三种匹配方式）
pub enum ServerMatcher {
    /// 按服务器名称匹配
    ByName(Regex),
    /// 按命令数组匹配（stdio）
    ByCommand(Vec<String>),
    /// 按 URL 模式匹配（远程服务器）
    ByUrl(Regex),
}

use regex::Regex;

impl EnterprisePolicyFilter {
    /// 创建企业策略过滤器
    pub fn new(denied: Vec<ServerMatcher>, allowed: Option<Vec<ServerMatcher>>) -> Self {
        Self { denied, allowed }
    }

    /// 过滤 MCP 服务器配置
    pub fn filter(&self, servers: Vec<McpServerConfig>) -> McpFilterResult {
        let mut result = McpFilterResult::default();

        for server in servers {
            // SDK 服务器豁免策略检查
            if server.r#type == "sdk" {
                result.allowed.push(server);
                continue;
            }

            // 黑名单检查
            if self.matches_any_denied(&server) {
                result
                    .denied
                    .push((server, "Blocked by enterprise denylist".to_string()));
                continue;
            }

            // 白名单检查
            if let Some(allowlist) = &self.allowed {
                if !self.matches_any_allowed(allowlist, &server) {
                    result
                        .denied
                        .push((server, "Not in enterprise allowlist".to_string()));
                    continue;
                }
            }

            result.allowed.push(server);
        }

        result
    }

    /// 检查是否匹配任何黑名单
    fn matches_any_denied(&self, server: &McpServerConfig) -> bool {
        self.denied
            .iter()
            .any(|matcher| self.matcher_matches(matcher, server))
    }

    /// 检查是否匹配任何白名单
    fn matches_any_allowed(&self, allowlist: &[ServerMatcher], server: &McpServerConfig) -> bool {
        allowlist
            .iter()
            .any(|matcher| self.matcher_matches(matcher, server))
    }

    /// 匹配器匹配检查
    fn matcher_matches(&self, matcher: &ServerMatcher, server: &McpServerConfig) -> bool {
        match matcher {
            ServerMatcher::ByName(pattern) => pattern.as_str() == server.name,
            ServerMatcher::ByCommand(cmd_list) => server
                .stdio_config
                .as_ref()
                .map(|cfg| cfg.command == cmd_list[0] && cfg.args == &cmd_list[1..])
                .unwrap_or(false),
            ServerMatcher::ByUrl(pattern) => server
                .remote_config
                .as_ref()
                .map(|cfg| pattern.is_match(&cfg.url))
                .unwrap_or(false),
        }
    }
}

/// MCP 过滤结果
#[derive(Debug, Default)]
pub struct McpFilterResult {
    /// 允许的服务器
    pub allowed: Vec<McpServerConfig>,
    /// 被拒绝的服务器
    pub denied: Vec<(McpServerConfig, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_matcher_by_name() {
        let pattern = Regex::new("^github$").unwrap();
        let matcher = ServerMatcher::ByName(pattern);

        let server = McpServerConfig {
            name: "github".to_string(),
            r#type: "stdio".to_string(),
            disabled: false,
            stdio_config: None,
            remote_config: None,
            sdk_config: None,
            scope: ConfigScope::Local,
        };

        let filter = EnterprisePolicyFilter::new(vec![matcher], None);
        assert!(filter.matches_any_denied(&server));
    }

    #[test]
    fn test_sdk_server_exemption() {
        let server = McpServerConfig {
            name: "sdk-server".to_string(),
            r#type: "sdk".to_string(),
            disabled: false,
            stdio_config: None,
            remote_config: None,
            sdk_config: Some(SdkConfig {
                register_fn: "my_fn".to_string(),
            }),
            scope: ConfigScope::Local,
        };

        // SDK 服务器应该被豁免
        let filter = EnterprisePolicyFilter::new(vec![], None);
        let result = filter.filter(vec![server]);
        assert_eq!(result.allowed.len(), 1);
        assert_eq!(result.allowed[0].r#type, "sdk");
    }
}
