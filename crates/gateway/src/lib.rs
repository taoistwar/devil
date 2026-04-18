//! Gateway crate - 提供 API 网关和 HTTP 接口
//!
//! 本 crate 负责提供外部访问 Agent 的 HTTP API 接口

use anyhow::Result;
use channels::{ChannelManager, Message};
use std::sync::Arc;
use tracing::{debug, error, info};

/// API 网关配置
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// 监听的地址
    pub host: String,
    /// 监听的端口
    pub port: u16,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

/// API 网关
pub struct Gateway {
    config: GatewayConfig,
    channel_manager: Arc<ChannelManager>,
}

impl Gateway {
    /// 创建新的网关实例
    pub fn new(config: GatewayConfig, channel_manager: Arc<ChannelManager>) -> Self {
        Self {
            config,
            channel_manager,
        }
    }

    /// 获取配置
    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    /// 发送消息到通道
    pub async fn send_message(&self, message: Message) -> Result<()> {
        debug!("Gateway sending message: {:?}", message);
        self.channel_manager.broadcast(message)?;
        Ok(())
    }

    /// 启动网关服务（简化版本，仅框架）
    pub async fn start(&self) -> Result<()> {
        info!(
            "Gateway would start on {}:{}",
            self.config.host, self.config.port
        );
        debug!("Channel manager ready");

        // TODO: 实现 HTTP 服务器
        // 可以使用 warp、axum 或 actix-web 等框架

        Ok(())
    }

    /// 停止网关服务
    pub async fn shutdown(&self) -> Result<()> {
        info!("Gateway shutting down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_creation() {
        let config = GatewayConfig::default();
        let channel_manager = Arc::new(ChannelManager::default());
        let gateway = Gateway::new(config, channel_manager);

        assert_eq!(gateway.config().port, 8080);
    }
}
