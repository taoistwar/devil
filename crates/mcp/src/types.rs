//! MCP 类型定义
//! 
//! 定义 MCP 服务器配置、连接状态、工具映射等核心类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// MCP 服务器配置（7 个作用域）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// 服务器名称
    pub name: String,
    
    /// 传输类型
    #[serde(rename = "type")]
    pub r#type: String,
    
    /// 是否禁用
    #[serde(default)]
    pub disabled: bool,
    
    /// stdio 专用字段（扁平化）
    #[serde(flatten)]
    pub stdio_config: Option<StdioConfig>,
    
    /// 远程服务器专用字段（扁平化）
    #[serde(flatten)]
    pub remote_config: Option<RemoteConfig>,
    
    /// SDK 专用字段（扁平化）
    #[serde(flatten)]
    pub sdk_config: Option<SdkConfig>,
    
    /// 配置来源作用域（跳过序列化）
    #[serde(skip)]
    pub scope: ConfigScope,
}

/// stdio 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioConfig {
    /// 启动命令
    pub command: String,
    /// 命令参数
    #[serde(default)]
    pub args: Vec<String>,
    /// 环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// 远程服务器配置（SSE/HTTP/WS）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// 服务器 URL
    pub url: String,
}

/// SDK 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    /// 注册函数名称
    pub register_fn: String,
}

/// 配置作用域（7 层）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConfigScope {
    #[default]
    /// 项目 - 个人（不提交到版本控制）
    Local,
    /// 项目 - 共享（团队配置）
    Project,
    /// 用户全局（跨项目使用）
    User,
    /// 运行时动态添加（会话级）
    Dynamic,
    /// 组织级（企业策略）
    Enterprise,
    /// 平台级（Claude.ai 连接器）
    Claudeai,
    /// 管理级（IT 管理员强制执行）
    Managed,
}

/// 服务器能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// 支持的工具
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
    /// 支持的模式
    #[serde(default)]
    pub prompts: Option<PromptsCapability>,
    /// 支持的资源
    #[serde(default)]
    pub resources: Option<ResourcesCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(default)]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(default)]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(default)]
    pub subscribe: Option<bool>,
    #[serde(default)]
    pub list_changed: Option<bool>,
}

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// 工具名称
    pub name: String,
    /// 工具描述
    #[serde(default)]
    pub description: String,
    /// 输入参数 Schema
    pub input_schema: JsonSchema,
    /// 行为注解
    #[serde(default)]
    pub hints: McpToolHints,
    /// 元数据（包含 Anthropic 扩展字段）
    #[serde(default, rename = "_meta")]
    pub meta: McpToolMeta,
    /// 服务器名称（由加载器注入）
    #[serde(skip)]
    pub server_name: String,
}

/// JSON Schema（简化的定义）
pub type JsonSchema = serde_json::Value;

/// 工具行为注解
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpToolHints {
    /// 只读提示 → 映射到 isConcurrencySafe()
    #[serde(rename = "readOnlyHint", default)]
    pub read_only: bool,
    /// 破坏性提示 → 映射到 isDestructive()
    #[serde(rename = "destructiveHint", default)]
    pub destructive: bool,
    /// 开放世界提示 → 映射到 isOpenWorld()
    #[serde(rename = "openWorldHint", default)]
    pub open_world: bool,
}

/// MCP 工具元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpToolMeta {
    /// Anthropic 扩展：是否始终加载
    #[serde(rename = "anthropic/alwaysLoad", default)]
    pub anthropic_always_load: Option<bool>,
}

/// 映射到内部 Tool 对象的 MCP 工具
#[derive(Debug, Clone)]
pub struct MappedTool {
    /// 工具名称（三段式：mcp__server__tool）
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入参数 Schema
    pub input_schema: JsonSchema,
    /// MCP 工具标记
    pub is_mcp: bool,
    /// MCP 元信息
    pub mcp_info: McpToolInfo,
    /// 行为注解桥接
    pub hints: ToolHints,
    /// 是否始终加载（跳过延迟加载）
    pub always_load: bool,
}

/// MCP 工具元信息
#[derive(Debug, Clone)]
pub struct McpToolInfo {
    /// 服务器名称
    pub server_name: String,
    /// 工具原始名称
    pub tool_name: String,
    /// 服务器类型
    pub server_type: String,
}

/// 行为注解桥接
#[derive(Debug, Clone, Default)]
pub struct ToolHints {
    /// 只读 → 并发安全
    pub read_only: bool,
    /// 破坏性
    pub destructive: bool,
    /// 开放世界
    pub open_world: bool,
}

/// 已连接的 MCP 服务器
pub struct ConnectedMcpServer {
    /// 服务器名称
    pub name: String,
    /// MCP Client 实例（占位符，实际实现需要 MCP Client crate）
    pub client: Box<dyn std::any::Any + Send>,
    /// 服务器能力声明
    pub capabilities: ServerCapabilities,
    /// 配置信息
    pub config: std::sync::Arc<McpServerConfig>,
    /// 清理函数（优雅断开连接）
    pub cleanup: Box<dyn FnOnce() + Send>,
}

unsafe impl Sync for ConnectedMcpServer {}

/// 等待重连的 MCP 服务器
pub struct PendingMcpServer {
    /// 服务器名称
    pub name: String,
    /// 重连尝试次数
    pub retry_count: usize,
    /// 最大重连次数
    pub max_retries: usize,
    /// 下次重试时间
    pub next_retry_time: Instant,
}

/// MCP 连接状态
pub enum McpConnectionState {
    /// 已连接，工具可用
    Connected {
        capabilities: ServerCapabilities,
        cleanup: Box<dyn FnOnce() + Send>,
    },
    /// 连接失败，记录原因
    Failed {
        reason: String,
        retryable: bool,
    },
    /// 需要认证
    NeedsAuth,
    /// 等待重连（指数退避）
    Pending {
        retry_count: usize,
        max_retries: usize,
        next_retry_time: Instant,
    },
    /// 已禁用
    Disabled,
}

impl Default for McpConnectionState {
    fn default() -> Self {
        McpConnectionState::Disabled
    }
}

unsafe impl Sync for McpConnectionState {}

/// MCP 加载结果（容错模式）
#[derive(Debug, Default)]
pub struct McpLoadResult {
    /// 成功加载的服务器
    pub allowed: Vec<McpServerConfig>,
    /// 被拒绝的服务器
    pub denied: Vec<(McpServerConfig, String)>,
    /// 加载错误
    pub errors: Vec<McpLoadError>,
}

/// MCP 加载错误
#[derive(Debug, thiserror::Error)]
pub enum McpLoadError {
    #[error("插件元数据缺失或无效：{0}")]
    InvalidMetadata(String),
    
    #[error("插件加载失败：{0}")]
    LoadFailed(String),
    
    #[error("安全检查失败：{0}")]
    SecurityCheckFailed(String),
    
    #[error("版本不兼容：{0}")]
    VersionMismatch(String),
}

impl McpConnectionState {
    /// 判断是否为可重试错误
    pub fn is_retryable_error(error: &str) -> bool {
        // 网络超时、连接拒绝等可以重试
        // 认证失败、权限拒绝等不可重试
        !error.contains("auth") && !error.contains("permission")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_scope_serialization() {
        let scope = ConfigScope::Local;
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, "\"local\"");
    }
    
    #[test]
    fn test_tool_hints_mapping() {
        let mcp_hints = McpToolHints {
            read_only: true,
            destructive: false,
            open_world: true,
        };
        
        let hints = ToolHints {
            read_only: mcp_hints.read_only,
            destructive: mcp_hints.destructive,
            open_world: mcp_hints.open_world,
        };
        
        assert!(hints.read_only);
        assert!(!hints.destructive);
        assert!(hints.open_world);
    }
}
