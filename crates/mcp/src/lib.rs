//! MCP (Model Context Protocol) 集成模块
//!
//! 实现 MCP 客户端功能，包括：
//! - 连接管理器与生命周期管理
//! - 8 种传输协议实现
//! - 工具发现、映射和注册
//! - 四层权限模型与安全策略
//! - Bridge 双向通信系统
//!
//! ## 架构概览
//!
//! MCP 是 AI 世界的"USB-C 接口"，定义了 AI 应用与外部数据源和工具之间的统一标准协议。
//! 通过 MCP，Devil Agent 可以以统一的方式连接任何支持 MCP 的服务器。
//!
//! ## 模块结构
//!
//! ```
//! mcp/
//! ├── lib.rs                  # 模块入口
//! ├── connection_manager.rs   # 连接管理器与状态机
//! ├── types.rs                # 类型定义
//! ├── transports/             # 8 种传输协议
//! │   ├── mod.rs
//! │   ├── stdio.rs
//! │   ├── sse.rs
//! │   ├── http.rs
//! │   ├── websocket.rs
//! │   └── sdk.rs
//! ├── tool_discovery.rs       # 工具发现与映射
//! ├── permissions.rs          # 四层权限模型
//! └── bridge/                 # Bridge 双向通信
//!     ├── mod.rs
//!     ├── router.rs
//!     └── control.rs
//! ```
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use devil_mcp::{McpConnectionManager, McpServerConfig, ConfigScope};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 创建连接管理器
//!     let manager = McpConnectionManager::new();
//!     
//!     // 从配置加载所有服务器
//!     let result = manager.load_from_config().await?;
//!     println!("Loaded {} MCP servers", result.allowed.len());
//!     
//!     // 重新连接指定服务器
//!     manager.reconnect("filesystem").await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod bridge;
pub mod connection_manager;
pub mod control_protocol;
pub mod permissions;
pub mod tool_discovery;
pub mod transports;
pub mod types;

pub use connection_manager::{McpConnection, McpConnectionManager};

pub use types::{ConnectedMcpServer, McpConnectionState, PendingMcpServer};

pub use types::{ConfigScope, McpServerConfig, ServerCapabilities};

pub use transports::{
    HttpPollingTransport, SdkTransport, StdioTransport, StreamableHttpTransport, Transport,
    TransportType, WebSocketTransport,
};

pub use tool_discovery::{
    clean_unicode, format_mcp_tool_name, parse_mcp_tool_name, MappedTool, McpTool, ToolDiscoverer,
    ToolStats,
};

pub use permissions::{
    EnterprisePolicy, IdeWhitelist, PermissionChecker, PermissionResult, UserPermissions,
};

pub use bridge::{
    BoundedUuidSet, BridgeMessage, BridgeState, JsonRpcError, McpBridge, MessageRouter,
};

pub use control_protocol::{
    build_initialize_request, parse_initialize_response, ClientInfo, ControlProtocol,
    ControlRequest, ControlResponse, ServerInfo,
};
