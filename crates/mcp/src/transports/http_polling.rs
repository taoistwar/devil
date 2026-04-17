//! HTTP Polling 传输协议实现
//!
//! 兼容旧版 MCP 服务器的轮询模式
//! 通过定时 HTTP 请求获取更新

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::Transport;

/// HTTP 轮询传输层
pub struct HttpPollingTransport {
    client: Client,
    base_url: String,
    poll_interval: std::time::Duration,
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl HttpPollingTransport {
    /// 创建新的 HTTP 轮询传输
    pub async fn connect(base_url: &str, poll_interval_ms: u64) -> Result<Self> {
        info!("Connecting to MCP server via HTTP polling: {}", base_url);

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let (tx, rx) = mpsc::channel::<String>(100);
        let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

        let transport = Self {
            client,
            base_url: base_url.to_string(),
            poll_interval: std::time::Duration::from_millis(poll_interval_ms),
            tx,
            rx,
            alive,
        };

        // 启动轮询后台任务
        transport.start_polling_loop();

        Ok(transport)
    }

    /// 启动轮询循环
    fn start_polling_loop(&self) {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let rx_for_poll = self.tx.clone();
        let alive = self.alive.clone();

        tokio::spawn(async move {
            loop {
                if !alive.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                // 轮询新消息
                let url = format!("{}/poll", base_url.trim_end_matches('/'));
                
                match client.get(&url).send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.text().await {
                                Ok(body) => {
                                    if !body.is_empty() && body != "null" {
                                        debug!("Polled message: {}", body);
                                        
                                        if rx_for_poll.send(body).await.is_err() {
                                            error!("Failed to send polled message to channel");
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to read poll response: {}", e);
                                }
                            }
                        } else if resp.status() == 204 {
                            // 204 No Content - 正常，无新消息
                        } else {
                            warn!("Poll returned non-200: {}", resp.status());
                        }
                    }
                    Err(e) => {
                        error!("Poll request failed: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }

                // 等待下次轮询
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }
}

#[async_trait]
impl Transport for HttpPollingTransport {
    async fn send(&self, message: String) -> Result<()> {
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            anyhow::bail!("Transport is not alive");
        }

        let url = format!("{}/send", self.base_url.trim_end_matches('/'));

        debug!("Sending HTTP POST: {}", url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(message)
            .send()
            .await
            .context("Failed to send HTTP request")?;

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
        info!("HTTP polling connection closed");
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(std::sync::atomic::Ordering::Relaxed)
    }
}
