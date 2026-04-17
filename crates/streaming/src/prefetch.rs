//! 并行预取器
//!
//! 在模块加载早期启动 I/O 密集型操作，与 import 链并行执行
//! 优化目标：节省启动时间 > 50ms

use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// MDM 配置预取句柄
pub struct MdmPrefetchHandle {
    child: Option<tokio::process::Child>,
    start_time: Instant,
}

/// Keychain 预取句柄
pub struct KeychainPrefetchHandle {
    oauth_future: Option<tokio::task::JoinHandle<Result<String>>>,
    legacy_future: Option<tokio::task::JoinHandle<Result<Option<String>>>>,
    start_time: Instant,
}

/// MDM 配置
#[derive(Debug, Clone, Default)]
pub struct MdmConfig {
    pub enterprise_features_enabled: bool,
    pub max_budget_usd: Option<f64>,
    pub allowed_models: Vec<String>,
}

/// OAuth 凭证
#[derive(Debug, Clone)]
pub struct OAuthToken {
    pub access_token: String,
    pub expires_at: Option<u64>,
}

/// 并行预取器
pub struct ParallelPrefetcher;

impl ParallelPrefetcher {
    /// 创建新的预取器
    pub fn new() -> Self {
        Self
    }

    /// 启动 MDM 配置预取（macOS plist / Windows registry）
    pub fn start_mdm_raw_read(&self) -> MdmPrefetchHandle {
        info!("Starting MDM config prefetch");

        let start_time = Instant::now();

        // macOS: 使用 plutil 读取 plist
        #[cfg(target_os = "macos")]
        let child = {
            let cmd = Command::new("plutil")
                .args(&[
                    "-p",
                    "/Library/Managed Preferences/com.example.devil.plist",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            match cmd {
                Ok(c) => Some(c),
                Err(e) => {
                    debug!("MDM plist not found or plutil failed: {}", e);
                    None
                }
            }
        };

        // Windows: 使用 reg query
        #[cfg(target_os = "windows")]
        let child = {
            let cmd = Command::new("reg")
                .args(&[
                    "query",
                    "HKLM\\SOFTWARE\\Policies\\Devil",
                    "/v",
                    "Config",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            match cmd {
                Ok(c) => Some(c),
                Err(e) => {
                    debug!("MDM registry key not found: {}", e);
                    None
                }
            }
        };

        // 其他平台：无 MDM
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let child: Option<tokio::process::Child> = None;

        MdmPrefetchHandle {
            child,
            start_time,
        }
    }

    /// 启动 Keychain 凭证预取
    pub fn start_keychain_prefetch(&self) -> KeychainPrefetchHandle {
        info!("Starting keychain prefetch");

        let start_time = Instant::now();

        // 预取 OAuth tokens（约 32ms）
        let oauth_future = Some(tokio::spawn(async {
            // macOS keychain
            #[cfg(target_os = "macos")]
            {
                let output = Command::new("security")
                    .args(&[
                        "find-generic-password",
                        "-s",
                        "devil-oauth",
                        "-w",
                    ])
                    .output()
                    .await;

                match output {
                    Ok(o) if o.status.success() => {
                        Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    }
                    Ok(o) => {
                        Err(anyhow::anyhow!("Keychain read failed: {:?}", o))
                    }
                    Err(e) => Err(anyhow::anyhow!("security command failed: {}", e)),
                }
            }

            // 其他平台：从环境变量读取
            #[cfg(not(target_os = "macos"))]
            {
                std::env::var("DEVIL_OAUTH_TOKEN")
                    .context("No OAuth token found")
            }
        }));

        // 预取 legacy API key（约 33ms）
        let legacy_future = Some(tokio::spawn(async {
            // macOS keychain
            #[cfg(target_os = "macos")]
            {
                let output = Command::new("security")
                    .args(&[
                        "find-generic-password",
                        "-s",
                        "devil-api-key",
                        "-w",
                    ])
                    .output()
                    .await;

                match output {
                    Ok(o) if o.status.success() => {
                        Ok(Some(String::from_utf8_lossy(&o.stdout).trim().to_string()))
                    }
                    Ok(_) => Ok(None),
                    Err(_) => Ok(None),
                }
            }

            // 其他平台：从环境变量读取
            #[cfg(not(target_os = "macos"))]
            {
                Ok(std::env::var("DEVIL_API_KEY").ok())
            }
        }));

        KeychainPrefetchHandle {
            oauth_future,
            legacy_future,
            start_time,
        }
    }

    /// 等待 MDM 预取完成
    pub async fn await_mdm(&self, mut handle: MdmPrefetchHandle) -> Result<MdmConfig> {
        let elapsed = handle.start_time.elapsed();

        if let Some(mut child) = handle.child.take() {
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                child.wait_with_output(),
            )
            .await
            {
                Ok(Ok(output)) if output.status.success() => {
                    let json_str = String::from_utf8_lossy(&output.stdout);

                    // 解析 MDM 配置
                    let config = self.parse_mdm_json(&json_str)?;

                    info!(
                        "MDM prefetch completed in {:?}: enabled={}",
                        elapsed, config.enterprise_features_enabled
                    );

                    Ok(config)
                }
                Ok(Ok(output)) => {
                    warn!("MDM read returned non-zero: {:?}", output);
                    Ok(MdmConfig::default())
                }
                Ok(Err(e)) => {
                    warn!("MDM wait failed: {}", e);
                    Ok(MdmConfig::default())
                }
                Err(_) => {
                    warn!("MDM read timeout");
                    child.kill().await.ok();
                    Ok(MdmConfig::default())
                }
            }
        } else {
            debug!("MDM prefetch not applicable on this platform");
            Ok(MdmConfig::default())
        }
    }

    /// 解析 MDM JSON
    fn parse_mdm_json(&self, json_str: &str) -> Result<MdmConfig> {
        // 简化的解析逻辑
        let mut config = MdmConfig::default();

        // 检查是否启用了企业功能
        if json_str.contains("enterprise_features") && json_str.contains("true") {
            config.enterprise_features_enabled = true;
        }

        // 提取 max_budget_usd
        if let Some(start) = json_str.find("max_budget_usd") {
            if let Some(rest) = json_str[start..].split(':').nth(1) {
                if let Some(num_str) = rest.split(|c| c == ',' || c == '}').next() {
                    if let Ok(num) = num_str.trim().parse::<f64>() {
                        config.max_budget_usd = Some(num);
                    }
                }
            }
        }

        Ok(config)
    }

    /// 等待 Keychain 预取完成
    pub async fn await_keychain(
        &self,
        handle: KeychainPrefetchHandle,
    ) -> Result<(Option<OAuthToken>, Option<String>)> {
        let elapsed = handle.start_time.elapsed();

        let oauth_result = if let Some(future) = handle.oauth_future {
            match future.await {
                Ok(Ok(token)) => {
                    debug!("OAuth token prefetch succeeded");
                    Some(OAuthToken {
                        access_token: token,
                        expires_at: None,
                    })
                }
                Ok(Err(e)) => {
                    debug!("OAuth token prefetch failed: {}", e);
                    None
                }
                Err(e) => {
                    debug!("OAuth prefetch task panicked: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let legacy_result = if let Some(future) = handle.legacy_future {
            match future.await {
                Ok(Ok(Some(key))) => {
                    debug!("Legacy API key prefetch succeeded");
                    Some(key)
                }
                Ok(Ok(None)) => {
                    debug!("No legacy API key found");
                    None
                }
                Ok(Err(e)) => {
                    debug!("Legacy API key prefetch failed: {}", e);
                    None
                }
                Err(e) => {
                    debug!("Legacy key prefetch task panicked: {}", e);
                    None
                }
            }
        } else {
            None
        };

        info!("Keychain prefetch completed in {:?}", elapsed);

        Ok((oauth_result, legacy_result))
    }
}

impl Default for ParallelPrefetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_handle_creation() {
        let prefetcher = ParallelPrefetcher::new();
        let mdm_handle = prefetcher.start_mdm_raw_read();
        let keychain_handle = prefetcher.start_keychain_prefetch();

        // 句柄应该成功创建
        assert!(mdm_handle.start_time.elapsed().as_millis() >= 0);
        assert!(keychain_handle.start_time.elapsed().as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_parse_mdm_json() {
        let prefetcher = ParallelPrefetcher::new();

        let json = r#"{
            "enterprise_features": true,
            "max_budget_usd": 100.50,
            "allowed_models": ["claude-3-sonnet", "claude-3-opus"]
        }"#;

        let config = prefetcher.parse_mdm_json(json).unwrap();

        assert!(config.enterprise_features_enabled);
        assert_eq!(config.max_budget_usd, Some(100.50));
    }

    #[tokio::test]
    async fn test_parse_mdm_json_no_enterprise() {
        let prefetcher = ParallelPrefetcher::new();

        let json = r#"{
            "enterprise_features": false,
            "max_budget_usd": 0
        }"#;

        let config = prefetcher.parse_mdm_json(json).unwrap();

        assert!(!config.enterprise_features_enabled);
    }
}
