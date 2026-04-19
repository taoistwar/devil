//! MCP Bridge 双向通信系统
//!
//! 负责 AI Agent 与 MCP 服务器之间的消息路由和通信
//! 实现：
//! - 请求 - 响应匹配（JSON-RPC ID）
//! - 通知广播
//! - 心跳和超时处理
//! - 去重（BoundedUUIDSet）

pub mod dedup;
pub mod message_router;

pub use dedup::BoundedUuidSet;
pub use message_router::MessageRouter;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Bridge 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeMessage {
    /// JSON-RPC 请求
    Request {
        id: serde_json::Value,
        method: String,
        params: serde_json::Value,
    },
    /// JSON-RPC 响应
    Response {
        id: serde_json::Value,
        result: Option<serde_json::Value>,
        error: Option<JsonRpcError>,
    },
    /// JSON-RPC 通知（无 ID）
    Notification {
        method: String,
        params: serde_json::Value,
    },
    /// 控制命令（initialize, interrupt 等）
    Control {
        command: String,
        payload: serde_json::Value,
    },
}

/// JSON-RPC 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Bridge 状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeState {
    /// 未连接
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 需要认证
    NeedsAuth,
    /// 错误
    Error(String),
}

/// MCP Bridge
pub struct McpBridge {
    /// 服务器 ID
    server_id: String,
    /// 消息路由器
    router: MessageRouter,
    /// 发送通道
    tx: mpsc::Sender<BridgeMessage>,
    #[allow(dead_code)]
    /// 接收通道
    rx: mpsc::Receiver<BridgeMessage>,
    /// 当前状态
    state: Arc<RwLock<BridgeState>>,
    /// 请求超时（毫秒）
    timeout_ms: u64,
}

impl McpBridge {
    /// 创建新的 Bridge
    pub fn new(server_id: &str, channel_size: usize, timeout_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel(channel_size);

        Self {
            server_id: server_id.to_string(),
            router: MessageRouter::new(),
            tx,
            rx,
            state: Arc::new(RwLock::new(BridgeState::Disconnected)),
            timeout_ms,
        }
    }

    /// 启动 Bridge
    pub async fn start(&self) -> Result<()> {
        info!("Starting MCP Bridge for server: {}", self.server_id);

        *self.state.write().await = BridgeState::Connecting;

        // 启动消息处理循环
        self.start_message_loop();

        Ok(())
    }

    /// 启动消息循环
    fn start_message_loop(&self) {
        // TODO: 由连接管理器统一管理
    }

    /// 发送 JSON-RPC 请求
    pub async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let id = uuid::Uuid::new_v4().to_string();

        debug!("Sending request {} (id: {}): {:?}", method, id, params);

        // 创建响应接收器
        let (response_tx, mut response_rx) = mpsc::channel::<BridgeMessage>(1);
        self.router.register_request(&id, response_tx).await;

        // 发送请求
        let msg = BridgeMessage::Request {
            id: serde_json::Value::String(id.clone()),
            method: method.to_string(),
            params,
        };

        self.tx.send(msg).await.context("Failed to send request")?;

        // 等待响应（带超时）
        match tokio::time::timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            response_rx.recv(),
        )
        .await
        {
            Ok(Some(BridgeMessage::Response { result, error, .. })) => {
                if let Some(err) = error {
                    anyhow::bail!("JSON-RPC error {}: {}", err.code, err.message);
                }
                Ok(result.unwrap_or(serde_json::Value::Null))
            }
            Ok(Some(other)) => {
                warn!("Unexpected message type: {:?}", other);
                anyhow::bail!("Unexpected response type");
            }
            Ok(None) => {
                anyhow::bail!("Response channel closed");
            }
            Err(_) => {
                anyhow::bail!("Request timeout ({}ms)", self.timeout_ms);
            }
        }
    }

    /// 发送通知（不等待响应）
    pub async fn send_notification(&self, method: &str, params: serde_json::Value) -> Result<()> {
        debug!("Sending notification {}: {:?}", method, params);

        let msg = BridgeMessage::Notification {
            method: method.to_string(),
            params,
        };

        self.tx
            .send(msg)
            .await
            .context("Failed to send notification")?;
        Ok(())
    }

    /// 发送控制命令
    pub async fn send_control(&self, command: &str, payload: serde_json::Value) -> Result<()> {
        debug!("Sending control command {}: {:?}", command, payload);

        let msg = BridgeMessage::Control {
            command: command.to_string(),
            payload,
        };

        self.tx.send(msg).await.context("Failed to send control")?;
        Ok(())
    }

    /// 发送 MCP initialize 请求
    pub async fn initialize(
        &self,
        protocol_version: &str,
        capabilities: serde_json::Value,
        client_info: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "protocolVersion": protocol_version,
            "capabilities": capabilities,
            "clientInfo": client_info,
        });

        self.send_request("initialize", params).await
    }

    /// 发送 tools/list 请求
    pub async fn list_tools(&self) -> Result<serde_json::Value> {
        self.send_request("tools/list", serde_json::Value::Null)
            .await
    }

    /// 发送 tools/call 请求
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments,
        });

        self.send_request("tools/call", params).await
    }

    /// 发送 resources/list 请求
    pub async fn list_resources(&self) -> Result<serde_json::Value> {
        self.send_request("resources/list", serde_json::Value::Null)
            .await
    }

    /// 发送 prompts/list 请求
    pub async fn list_prompts(&self) -> Result<serde_json::Value> {
        self.send_request("prompts/list", serde_json::Value::Null)
            .await
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> BridgeState {
        self.state.read().await.clone()
    }

    /// 更新状态
    pub async fn set_state(&self, state: BridgeState) {
        let mut current = self.state.write().await;
        debug!("Bridge state changed: {:?} -> {:?}", *current, state);
        *current = state;
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == BridgeState::Connected
    }

    /// 关闭 Bridge
    pub async fn close(&self) -> Result<()> {
        info!("Closing MCP Bridge for server: {}", self.server_id);

        self.set_state(BridgeState::Disconnected).await;
        self.router.clear().await;

        Ok(())
    }
}
