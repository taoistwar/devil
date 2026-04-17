//! Stdio 传输协议实现
//!
//! 通过标准输入输出与子进程通信，适用于本地 MCP 服务器

use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::Transport;

/// Stdio 传输层
pub struct StdioTransport {
    /// 子进程句柄
    child: Child,
    /// 发送通道
    tx: mpsc::Sender<String>,
    /// 是否存活标志
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl StdioTransport {
    /// 创建新的 Stdio 传输
    pub async fn spawn(command: &str, args: &[String]) -> Result<Self> {
        info!("Spawning MCP server: {} {:?}", command, args);

        let mut child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn MCP server process")?;

        let stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let stderr = child.stderr.take().context("Failed to open stderr")?;

        let (tx, mut rx) = mpsc::channel::<String>(100);
        let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let alive_clone = alive.clone();

        // 处理 stderr 输出（日志）
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                let line_trimmed = line.trim();
                debug!("MCP server stderr: {}", line_trimmed);
                line.clear();
                
                if !alive_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
            }
        });

        // 写入循环：从通道读取消息并写入子进程 stdin
        let mut stdin_writer = stdin;
        let alive_write = alive.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if !alive_write.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let mut full_msg = msg.clone();
                full_msg.push('\n');
                
                if let Err(e) = stdin_writer.write_all(full_msg.as_bytes()).await {
                    error!("Failed to write to MCP server stdin: {}", e);
                    alive_write.store(false, std::sync::atomic::Ordering::Relaxed);
                    break;
                }

                if let Err(e) = stdin_writer.flush().await {
                    error!("Failed to flush MCP server stdin: {}", e);
                    alive_write.store(false, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
            }
        });

        // 读取循环：从子进程 stdout 读取消息（由外部管理处理）
        let _stdout_reader = stdout; // 在 recv 方法中处理

        Ok(Self {
            child,
            tx,
            alive,
        })
    }

    /// 内部读取消息
    pub async fn read_line(&self) -> Result<Option<String>> {
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(None);
        }

        let stdout = self.child.stdout.as_ref().context("No stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF，进程结束
                self.alive.store(false, std::sync::atomic::Ordering::Relaxed);
                Ok(None)
            }
            Ok(_) => {
                let line_trimmed = line.trim().to_string();
                debug!("MCP server stdout: {}", line_trimmed);
                Ok(Some(line_trimmed))
            }
            Err(e) => {
                error!("Failed to read from MCP server stdout: {}", e);
                self.alive.store(false, std::sync::atomic::Ordering::Relaxed);
                Err(e.into())
            }
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&self, message: String) -> Result<()> {
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            anyhow::bail!("Transport is not alive");
        }

        self.tx.send(message).await.context("Failed to send message")?;
        Ok(())
    }

    async fn recv(&self) -> Result<Option<String>> {
        self.read_line().await
    }

    async fn close(&self) -> Result<()> {
        self.alive.store(false, std::sync::atomic::Ordering::Relaxed);
        
        // 尝试优雅关闭
        let mut child = self.child.try_clone().context("Failed to clone child")?;
        
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            
            let pid = child.id().context("No PID")?;
            kill(Pid::from_raw(pid as i32), Signal::SIGTERM).ok();
            
            // 等待 5 秒，然后强制杀死
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                child.wait()
            ).await.ok();
        }

        // 确保进程终止
        child.kill().await.ok();
        
        info!("MCP server process terminated");
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        self.alive.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
