//! MCP 传输协议实现
//!
//! 支持 8 种传输协议：
//! - stdio: 标准输入输出（本地进程）
//! - streamable-http: 双工 SSE（推荐）
//! - http-polling: HTTP 轮询（兼容旧服务器）
//! - websocket: WebSocket 全双工
//! - mcpcli-rust: Rust SDK（同进程）
//! - mcpcli-python: Python SDK（同进程）
//! - mcpcli-node: Node.js SDK（同进程）
//! - mcpcli-bun: Bun SDK（同进程）

pub mod stdio;
pub mod streamable_http;
pub mod http_polling;
pub mod websocket;
pub mod sdk;

pub use stdio::StdioTransport;
pub use streamable_http::StreamableHttpTransport;
pub use http_polling::HttpPollingTransport;
pub use websocket::WebSocketTransport;
pub use sdk::SdkTransport;

use anyhow::Result;
use async_trait::async_trait;

/// 传输层 trait
///
/// 所有传输协议必须实现此 trait，提供统一的 JSON-RPC 消息收发接口
#[async_trait]
pub trait Transport: Send + Sync {
    /// 发送 JSON-RPC 请求
    async fn send(&self, message: String) -> Result<()>;

    /// 接收 JSON-RPC 响应（返回 None 表示连接关闭）
    async fn recv(&mut self) -> Result<Option<String>>;

    /// 关闭连接
    async fn close(&self) -> Result<()>;

    /// 检查连接是否存活
    fn is_alive(&self) -> bool;
}

/// 传输协议类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportType {
    Stdio,
    StreamableHttp,
    HttpPolling,
    WebSocket,
    SdkRust,
    SdkPython,
    SdkNode,
    SdkBun,
}

impl TransportType {
    /// 从服务器配置判断传输类型
    pub fn from_config(config: &crate::McpServerConfig) -> Self {
        // SDK 类型优先判断
        if let Some(sdk_config) = &config.sdk_config {
            if !sdk_config.register_fn.is_empty() {
                return Self::SdkRust; // Simplified - would need sdk_language field
            }
        }

        // 根据 URL 或命令判断协议类型
        if let Some(remote_config) = &config.remote_config {
            let url = &remote_config.url;
            if url.starts_with("ws://") || url.starts_with("wss://") {
                Self::WebSocket
            } else if url.contains("/sse") || url.contains("/mcp") {
                Self::StreamableHttp
            } else {
                Self::HttpPolling
            }
        } else {
            // 默认为 stdio
            Self::Stdio
        }
    }
}
