//! MCP 控制协议实现
//!
//! 实现 MCP 控制消息的处理，包括：
//! - initialize - 初始化连接
//! - set_model - 切换模型
//! - interrupt - 中断当前操作
//! - ping/pong - 心跳检测
//! - cancel - 取消请求

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

/// 控制请求类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum ControlRequest {
    /// 初始化 MCP 连接
    #[serde(rename = "initialize")]
    Initialize {
        protocol_version: String,
        capabilities: Value,
        client_info: ClientInfo,
    },
    /// 设置模型
    #[serde(rename = "set_model")]
    SetModel {
        model_id: String,
    },
    /// 中断当前操作
    #[serde(rename = "interrupt")]
    Interrupt {
        reason: Option<String>,
    },
    /// 取消指定请求
    #[serde(rename = "cancel")]
    Cancel {
        request_id: String,
    },
    /// Ping 心跳
    #[serde(rename = "ping")]
    Ping,
    /// Pong 响应
    #[serde(rename = "pong")]
    Pong,
}

/// 客户端信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// 控制响应类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ControlResponse {
    /// 初始化成功
    InitializeSuccess {
        protocol_version: String,
        capabilities: Value,
        server_info: ServerInfo,
    },
    /// 模型设置成功
    ModelSet {
        model_id: String,
    },
    /// 中断成功
    Interrupted {
        reason: Option<String>,
    },
    /// 取消成功
    Cancelled {
        request_id: String,
    },
    /// Pong 响应
    Pong,
    /// 错误
    Error {
        code: i32,
        message: String,
    },
}

/// 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// 控制协议处理器
pub struct ControlProtocol;

impl ControlProtocol {
    /// 处理初始化请求
    pub async fn handle_initialize(
        request: ControlRequest,
    ) -> Result<ControlResponse> {
        if let ControlRequest::Initialize {
            protocol_version,
            capabilities,
            client_info,
        } = request
        {
            info!(
                "Initializing MCP connection: client={}:{} protocol={}",
                client_info.name, client_info.version, protocol_version
            );

            // 验证协议版本
            if !is_supported_version(&protocol_version) {
                return Ok(ControlResponse::Error {
                    code: -32602,
                    message: format!("Unsupported protocol version: {}", protocol_version),
                });
            }

            // 返回服务器能力
            let server_capabilities = serde_json::json!({
                "tools": {},
                "resources": {},
                "prompts": {},
            });

            Ok(ControlResponse::InitializeSuccess {
                protocol_version: protocol_version.clone(),
                capabilities: server_capabilities,
                server_info: ServerInfo {
                    name: "devil-mcp".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            })
        } else {
            Err(anyhow::anyhow!("Expected initialize request"))
        }
    }

    /// 处理设置模型请求
    pub async fn handle_set_model(model_id: String) -> Result<ControlResponse> {
        info!("Setting model: {}", model_id);

        // TODO: 实际切换模型逻辑
        debug!("Model switched to: {}", model_id);

        Ok(ControlResponse::ModelSet { model_id })
    }

    /// 处理中断请求
    pub async fn handle_interrupt(reason: Option<String>) -> Result<ControlResponse> {
        warn!("Interrupt requested: {:?}", reason);

        // TODO: 实际中断逻辑

        Ok(ControlResponse::Interrupted { reason })
    }

    /// 处理取消请求
    pub async fn handle_cancel(request_id: String) -> Result<ControlResponse> {
        info!("Cancelling request: {}", request_id);

        // TODO: 实际取消逻辑

        Ok(ControlResponse::Cancelled { request_id })
    }

    /// 处理 Ping 请求
    pub async fn handle_ping() -> Result<ControlResponse> {
        debug!("Received ping");
        Ok(ControlResponse::Pong)
    }

    /// 解析控制请求
    pub fn parse_request(json: &str) -> Result<ControlRequest> {
        serde_json::from_str(json).context("Failed to parse control request")
    }

    /// 序列化控制响应
    pub fn serialize_response(response: &ControlResponse) -> String {
        serde_json::to_string(response).unwrap_or_default()
    }
}

/// 检查协议版本是否支持
fn is_supported_version(version: &str) -> bool {
    // 支持的版本列表
    let supported = ["2024-11-05", "2024-10-07", "1.0.0", "2025-01-01"];
    supported.contains(&version)
}

/// 构建 initialize 请求
pub fn build_initialize_request(
    client_name: &str,
    client_version: &str,
    protocol_version: &str,
) -> Value {
    serde_json::json!({
        "protocolVersion": protocol_version,
        "capabilities": {
            "roots": {
                "listChanged": true
            },
            "sampling": {}
        },
        "clientInfo": {
            "name": client_name,
            "version": client_version
        }
    })
}

/// 解析 initialize 响应
pub fn parse_initialize_response(json: &Value) -> Result<ServerInfo> {
    let server_info = json
        .get("serverInfo")
        .context("Missing serverInfo in response")?;

    let name = server_info
        .get("name")
        .and_then(Value::as_str)
        .context("Missing server name")?
        .to_string();

    let version = server_info
        .get("version")
        .and_then(Value::as_str)
        .context("Missing server version")?
        .to_string();

    Ok(ServerInfo { name, version })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_request() {
        let request = ControlRequest::Initialize {
            protocol_version: "2024-11-05".to_string(),
            capabilities: serde_json::json!({}),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let response = ControlProtocol::handle_initialize(request).await.unwrap();

        match response {
            ControlResponse::InitializeSuccess { protocol_version, .. } => {
                assert_eq!(protocol_version, "2024-11-05");
            }
            _ => panic!("Expected InitializeSuccess"),
        }
    }

    #[tokio::test]
    async fn test_unsupported_version() {
        let request = ControlRequest::Initialize {
            protocol_version: "unsupported".to_string(),
            capabilities: serde_json::json!({}),
            client_info: ClientInfo {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let response = ControlProtocol::handle_initialize(request).await.unwrap();

        match response {
            ControlResponse::Error { message, .. } => {
                assert!(message.contains("Unsupported protocol version"));
            }
            _ => panic!("Expected Error"),
        }
    }

    #[tokio::test]
    async fn test_set_model() {
        let response = ControlProtocol::handle_set_model("gpt-4".to_string())
            .await
            .unwrap();

        match response {
            ControlResponse::ModelSet { model_id } => {
                assert_eq!(model_id, "gpt-4");
            }
            _ => panic!("Expected ModelSet"),
        }
    }

    #[tokio::test]
    async fn test_ping_pong() {
        let response = ControlProtocol::handle_ping().await.unwrap();

        match response {
            ControlResponse::Pong => {}
            _ => panic!("Expected Pong"),
        }
    }

    #[test]
    fn test_build_initialize_request() {
        let req = build_initialize_request("devil", "0.1.0", "2024-11-05");

        assert_eq!(req["protocolVersion"], "2024-11-05");
        assert_eq!(req["clientInfo"]["name"], "devil");
        assert_eq!(req["clientInfo"]["version"], "0.1.0");
    }
}
