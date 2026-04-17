//! Streamable HTTP 传输协议实现（双工 SSE）
//!
//! MCP 推荐协议：POST 发送请求，SSE 接收响应
//! 支持 HTTP/2 多路复用，单个连接可并发处理多个请求

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::Transport;

/// Streamable HTTP 传输层
pub struct StreamableHttpTransport {
    /// HTTP 客户端
    client: Client,
    /// 服务器基础 URL
    base_url: String,
    /// SSE 会话 ID（如果已建立）
    session_id: Mutex<Option<String>>,
    /// 发送通道
    tx: mpsc::Sender<String>,
    /// 接收通道
    rx: mpsc::Receiver<String>,
    /// 是否存活
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl StreamableHttpTransport {
    /// 创建新的 Streamable HTTP 传输
    pub async fn connect(base_url: &str) -> Result<Self> {
        info!("Connecting to MCP server via Streamable HTTP: {}", base_url);

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let (tx, rx) = mpsc::channel::<String>(100);
        let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

        let transport = Self {
            client,
            base_url: base_url.to_string(),
            session_id: Mutex::new(None),
            tx,
            rx,
            alive,
        };

        // 尝试初始化连接（发送 capability 请求）
        transport.initialize().await?;

        Ok(transport)
    }

    /// 初始化连接
    async fn initialize(&self) -> Result<()> {
        // 发送 OPTIONS 请求探测服务器能力
        let url = format!("{}/capabilities", self.base_url.trim_end_matches('/'));
        
        debug!("Probing MCP capabilities: {}", url);
        
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    debug!("MCP server capabilities probe succeeded");
                } else {
                    warn!("MCP capabilities probe returned non-200: {}", resp.status());
                }
            }
            Err(e) => {
                warn!("MCP capabilities probe failed (may be normal): {}", e);
            }
        }

        Ok(())
    }

    /// 处理 SSE 流
    async fn handle_sse_stream(&self, session_id: &str) -> Result<()> {
        let url = format!("{}/sse/{}", self.base_url.trim_end_matches('/'), session_id);
        
        debug!("Starting SSE stream: {}", url);

        loop {
            if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            let request = self.client.get(&url);
            
            match request.send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        error!("SSE stream failed with status: {}", resp.status());
                        self.alive.store(false, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }

                    // 处理 SSE 事件流
                    let bytes = match resp.bytes().await {
                        Ok(b) => b,
                        Err(e) => {
                            error!("Failed to read SSE response body: {}", e);
                            break;
                        }
                    };

                    let chunk_str = String::from_utf8_lossy(&bytes);
                    
                    // 简化的 SSE 解析（实际应使用标准库）
                    let mut event_data = String::new();
                    for line in chunk_str.lines() {
                        if line.starts_with("data: ") {
                            event_data.push_str(&line[6..]);
                        } else if line.is_empty() && !event_data.is_empty() {
                            // 空行表示事件结束
                            debug!("Received SSE event: {}", event_data);
                            
                            // 发送到接收通道
                            if self.tx.send(event_data.clone()).await.is_err() {
                                error!("Failed to send SSE event to channel");
                                break;
                            }
                            
                            event_data.clear();
                        }
                    }
                }
                Err(e) => {
                    error!("SSE connection error: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Transport for StreamableHttpTransport {
    async fn send(&self, message: String) -> Result<()> {
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            anyhow::bail!("Transport is not alive");
        }

        // 构建 POST 请求 URL
        let url = if let Some(session_id) = self.session_id.lock().unwrap().as_ref() {
            format!("{}/message/{}", self.base_url.trim_end_matches('/'), session_id)
        } else {
            self.base_url.clone()
        };

        debug!("Sending HTTP POST: {}", url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(message)
            .send()
            .await
            .context("Failed to send HTTP request")?;

        // 处理响应
        if let Some(session_id) = response.headers().get("Mcp-Session-Id") {
            let new_session_id = session_id.to_str().unwrap_or_default().to_string();
            
            if self.session_id.lock().unwrap().as_ref() != Some(&new_session_id) {
                debug!("New session ID: {}", new_session_id);
                
                // 启动 SSE 流处理（在新的 spawn 中）
                let transport_clone = self.clone_transport();
                let session_for_spawn = new_session_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = transport_clone.handle_sse_stream(&session_for_spawn).await {
                        error!("SSE stream handler failed: {}", e);
                    }
                });
                
                *self.session_id.lock().unwrap() = Some(new_session_id);
            }
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("HTTP request failed: {} - {}", status, body);
            anyhow::bail!("HTTP request failed: {} - {}", status, body);
        }

        Ok(())
    }

    async fn recv(&mut self) -> Result<Option<String>> {
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(None);
        }

        // 从接收通道获取消息
        match self.rx.recv().await {
            Some(msg) => Ok(Some(msg)),
            None => {
                self.alive.store(false, std::sync::atomic::Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn close(&self) -> Result<()> {
        self.alive.store(false, std::sync::atomic::Ordering::Relaxed);

        // 发送 DELETE 请求关闭会话
        let url_to_close = {
            if let Some(session_id) = self.session_id.lock().unwrap().as_ref() {
                Some(format!("{}/message/{}", self.base_url.trim_end_matches('/'), session_id))
            } else {
                None
            }
        };
        
        if let Some(url) = url_to_close {
            debug!("Closing SSE session: {}", url);
            self.client
                .delete(&url)
                .send()
                .await
                .ok();
        }

        info!("Streamable HTTP connection closed");
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl StreamableHttpTransport {
    fn clone_transport(&self) -> Self {
        let (tx, rx) = mpsc::channel::<String>(100);
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            session_id: Mutex::new(self.session_id.lock().unwrap().clone()),
            tx,
            rx,
            alive: self.alive.clone(),
        }
    }
}
