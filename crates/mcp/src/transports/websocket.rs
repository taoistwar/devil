//! WebSocket 传输协议实现
//!
//! 全双工通信，适用于实时 MCP 服务器

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::Transport;

/// WebSocket 传输层
pub struct WebSocketTransport {
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl WebSocketTransport {
    /// 创建新的 WebSocket 传输
    pub async fn connect(url: &str) -> Result<Self> {
        info!("Connecting to MCP server via WebSocket: {}", url);

        let (ws_stream, _) = connect_async(url)
            .await
            .context("Failed to connect WebSocket")?;

        let (write_tx, mut write_rx) = mpsc::channel::<String>(100);
        let (read_tx, read_rx) = mpsc::channel::<String>(100);
        let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // 写入循环：发送消息到 WebSocket
        let alive_write = alive.clone();
        tokio::spawn(async move {
            while let Some(msg) = write_rx.recv().await {
                if !alive_write.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                if let Err(e) = ws_sender.send(Message::Text(msg)).await {
                    error!("Failed to send WebSocket message: {}", e);
                    alive_write.store(false, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
            }

            // 关闭 WebSocket
            ws_sender.close().await.ok();
        });

        // 读取循环：从 WebSocket 接收消息
        let alive_read = alive.clone();
        tokio::spawn(async move {
            loop {
                if !alive_read.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                match ws_receiver.next().await {
                    Some(Ok(Message::Text(text))) => {
                        debug!("Received WebSocket message: {}", text);

                        if read_tx.send(text).await.is_err() {
                            error!("Failed to forward WebSocket message");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket closed by server");
                        alive_read.store(false, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }
                    Some(Ok(other)) => {
                        debug!("Received non-text WebSocket message: {:?}", other);
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        alive_read.store(false, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        alive_read.store(false, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            tx: write_tx,
            rx: read_rx,
            alive,
        })
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn send(&self, message: String) -> Result<()> {
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            anyhow::bail!("Transport is not alive");
        }

        self.tx
            .send(message)
            .await
            .context("Failed to send message")?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<Option<String>> {
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(None);
        }

        match self.rx.recv().await {
            Some(msg) => Ok(Some(msg)),
            None => {
                self.alive
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn close(&self) -> Result<()> {
        self.alive
            .store(false, std::sync::atomic::Ordering::Relaxed);
        info!("WebSocket connection closed");
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(std::sync::atomic::Ordering::Relaxed)
    }
}
