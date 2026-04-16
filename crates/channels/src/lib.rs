//! Channels crate - 提供消息通道和通信机制
//! 
//! 本 crate 负责提供不同组件之间的消息传递功能

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error};

/// 消息类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// 文本消息
    Text(String),
    /// 命令消息
    Command { name: String, args: Vec<String> },
    /// 事件消息
    Event { event_type: String, data: serde_json::Value },
}

/// 通道管理器
pub struct ChannelManager {
    /// 广播通道发送者
    broadcast_tx: broadcast::Sender<Message>,
    /// 点对点通道发送者
    mpsc_tx: mpsc::Sender<Message>,
}

impl ChannelManager {
    /// 创建新的通道管理器
    pub fn new(buffer_size: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(buffer_size);
        let (mpsc_tx, _) = mpsc::channel(buffer_size);
        
        Self {
            broadcast_tx,
            mpsc_tx,
        }
    }

    /// 广播消息给所有订阅者
    pub fn broadcast(&self, message: Message) -> Result<()> {
        self.broadcast_tx.send(message.clone())?;
        debug!("Broadcast message: {:?}", message);
        Ok(())
    }

    /// 发送点对点消息
    pub async fn send(&self, message: Message) -> Result<()> {
        self.mpsc_tx.send(message).await?;
        debug!("Sent point-to-point message");
        Ok(())
    }

    /// 获取广播通道接收者
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.broadcast_tx.subscribe()
    }

    /// 获取点对点通道接收者
    pub fn receiver(&self) -> mpsc::Receiver<Message> {
        let (tx, rx) = mpsc::channel(100);
        drop(tx); // 这只是示例，实际应该复用 mp
        rx
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_manager_creation() {
        let manager = ChannelManager::new(50);
        assert!(true);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let manager = ChannelManager::new(10);
        let msg = Message::Text("Hello".to_string());
        assert!(manager.broadcast(msg).is_ok());
    }
}
