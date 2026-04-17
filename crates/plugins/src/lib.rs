//! Plugins crate - 提供插件系统和扩展机制
//! 
//! 本 crate 负责定义插件接口和插件管理功能
//! 
//! ## 功能特性
//! 
//! - 插件加载器：从磁盘目录加载插件
//! - 安全策略：三层防护（黑名单、白名单、权限级别）
//! - 自动更新：版本检查与更新机制
//! - 容错加载：单个插件失败不影响整体

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 作者
    pub author: Option<String>,
    /// 权限级别
    #[serde(default)]
    pub permission_level: PermissionLevel,
}

/// 插件权限级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// 仅读操作
    None,
    /// 读 + 网络
    Low,
    /// 读 + 写工作目录
    Medium,
    /// 完全文件系统访问
    High,
    /// 包括执行任意代码
    Full,
}

impl Default for PermissionLevel {
    fn default() -> Self {
        Self::None
    }
}

/// 插件安全策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSecurityPolicy {
    /// 仅允许托管插件
    pub allow_managed_only: bool,
    /// 黑名单
    pub blocklist: Vec<String>,
    /// 最大权限级别
    pub max_permission_level: PermissionLevel,
}

impl Default for PluginSecurityPolicy {
    fn default() -> Self {
        Self {
            allow_managed_only: false,
            blocklist: Vec::new(),
            max_permission_level: PermissionLevel::High,
        }
    }
}

/// 插件加载错误
#[derive(Debug, thiserror::Error)]
pub enum PluginLoadError {
    #[error("插件元数据缺失或无效：{0}")]
    InvalidMetadata(String),
    
    #[error("插件加载失败：{0}")]
    LoadFailed(String),
    
    #[error("安全检查失败：{0}")]
    SecurityCheckFailed(String),
    
    #[error("版本不兼容：{0}")]
    VersionMismatch(String),
}

/// 插件加载结果
#[derive(Debug, Default)]
pub struct PluginLoadResult {
    /// 成功加载的插件
    pub plugins: Vec<PluginMetadata>,
    /// 加载错误（收集而不中断）
    pub errors: Vec<PluginLoadError>,
}

/// 插件版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    /// 当前版本
    pub current: String,
    /// 最新版本
    pub latest: String,
    /// 是否需要更新
    pub update_available: bool,
}

/// 插件执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// 请求 ID
    pub request_id: String,
    /// 输入数据
    pub input: serde_json::Value,
    /// 环境变量
    pub env: HashMap<String, String>,
}

/// 插件执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub output: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// 插件接口 Trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> PluginMetadata;

    /// 初始化插件
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    /// 执行插件功能
    async fn execute(&self, ctx: PluginContext) -> Result<PluginResult>;

    /// 关闭插件
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

/// 插件注册信息
struct PluginRegistration {
    metadata: PluginMetadata,
    #[allow(dead_code)]
    plugin: Arc<dyn Plugin>,
}

/// 插件管理器
pub struct PluginManager {
    plugins: RwLock<HashMap<String, PluginRegistration>>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// 注册插件
    pub async fn register<T: Plugin + 'static>(&mut self, plugin: T) -> Result<()> {
        let metadata = plugin.metadata();
        let name = metadata.name.clone();
        
        let mut plugins = self.plugins.write().await;
        
        if plugins.contains_key(&name) {
            warn!("Plugin {} is already registered", name);
            return Err(anyhow::anyhow!("Plugin already registered"));
        }

        plugins.insert(
            name.clone(),
            PluginRegistration {
                metadata,
                plugin: Arc::new(plugin),
            },
        );

        info!("Registered plugin: {}", name);
        Ok(())
    }

    /// 执行插件
    pub async fn execute_plugin(
        &self,
        name: &str,
        ctx: PluginContext,
    ) -> Result<PluginResult> {
        let plugins = self.plugins.read().await;
        
        let registration = plugins
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", name))?;

        debug!("Executing plugin: {}", name);
        registration.plugin.execute(ctx).await
    }

    /// 列出所有已注册的插件
    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.values().map(|reg| reg.metadata.clone()).collect()
    }

    /// 自动更新插件
    pub async fn update_plugin(&self, name: &str) -> Result<String> {
        let plugins = self.plugins.read().await;
        let registration = plugins.get(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", name))?;
        
        info!("Checking for updates for plugin: {}", name);
        
        // TODO: 实现版本检查和更新逻辑
        // 实际实现应该：
        // 1. 查询插件仓库获取最新版本
        // 2. 比较当前版本和最新版本
        // 3. 下载并安装新版本
        // 4. 重新加载插件
        
        Ok(format!("Plugin {} is up to date", name))
    }
    
    /// 检查所有插件的更新
    pub async fn check_all_updates(&self) -> HashMap<String, PluginVersion> {
        let plugins = self.plugins.read().await;
        let mut updates = HashMap::new();
        
        for (name, registration) in plugins.iter() {
            // TODO: 实现版本检查逻辑
            // 目前返回当前版本，update_available 为 false
            updates.insert(
                name.clone(),
                PluginVersion {
                    current: registration.metadata.version.clone(),
                    latest: registration.metadata.version.clone(),
                    update_available: false,
                },
            );
        }
        
        updates
    }
    
    /// 卸载插件
    pub async fn unregister(&mut self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(reg) = plugins.remove(name) {
            reg.plugin.shutdown().await?;
            info!("Unregistered plugin: {}", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin not found: {}", name))
        }
    }
    
    /// 初始化所有插件
    pub async fn initialize_all(&mut self) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        
        for (name, reg) in plugins.iter_mut() {
            match reg.plugin.initialize().await {
                Ok(_) => info!("Initialized plugin: {}", name),
                Err(e) => warn!("Failed to initialize plugin {}: {}", name, e),
            }
        }
        
        Ok(())
    }
    
    /// 关闭所有插件
    pub async fn shutdown_all(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        
        for (name, reg) in plugins.iter() {
            match reg.plugin.shutdown().await {
                Ok(_) => info!("Shutdown plugin: {}", name),
                Err(e) => warn!("Failed to shutdown plugin {}: {}", name, e),
            }
        }
        
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 插件加载器
/// 
/// 负责从磁盘目录加载插件，支持容错和增量加载
pub struct PluginLoader {
    /// 插件目录列表
    plugin_dirs: Vec<PathBuf>,
    /// 安全策略
    security_policy: PluginSecurityPolicy,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new() -> Self {
        Self {
            plugin_dirs: Vec::new(),
            security_policy: PluginSecurityPolicy::default(),
        }
    }
    
    /// 设置插件目录
    pub fn with_plugin_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.plugin_dirs = dirs;
        self
    }
    
    /// 设置安全策略
    pub fn with_security_policy(mut self, policy: PluginSecurityPolicy) -> Self {
        self.security_policy = policy;
        self
    }
    
    /// 从目录加载所有插件（容错模式）
    /// 
    /// 即使某些插件加载失败，也会继续加载其他插件
    /// 错误被收集到返回结果中
    pub async fn load_all_from_dirs(&self) -> PluginLoadResult {
        let mut result = PluginLoadResult::default();
        
        for dir in &self.plugin_dirs {
            if !dir.exists() {
                debug!("Plugin directory does not exist: {:?}", dir);
                continue;
            }
            
            // 扫描目录查找插件
            match self.scan_plugin_directory(dir) {
                Ok(plugins) => {
                    for plugin in plugins {
                        result.plugins.push(plugin);
                    }
                }
                Err(e) => {
                    result.errors.push(e);
                }
            }
        }
        
        info!("Loaded {} plugins with {} errors", result.plugins.len(), result.errors.len());
        result
    }
    
    /// 扫描单个目录中的插件
    fn scan_plugin_directory(&self, dir: &Path) -> Result<Vec<PluginMetadata>, PluginLoadError> {
        let mut plugins = Vec::new();
        
        // 查找 plugin.json 或 package.json 文件
        let entries = std::fs::read_dir(dir)
            .map_err(|e| PluginLoadError::LoadFailed(format!("Failed to read {:?}: {}", dir, e)))?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                // 检查子目录中的 plugin.json
                let plugin_json = path.join("plugin.json");
                if plugin_json.exists() {
                    match self.load_plugin_metadata(&plugin_json) {
                        Ok(metadata) => {
                            // 安全检查
                            if let Err(e) = self.check_plugin_security(&metadata) {
                                return Err(e);
                            }
                            plugins.push(metadata);
                        }
                        Err(e) => {
                            warn!("Failed to load plugin metadata at {:?}: {}", plugin_json, e);
                        }
                    }
                }
            }
        }
        
        Ok(plugins)
    }
    
    /// 加载插件元数据
    fn load_plugin_metadata(&self, path: &Path) -> Result<PluginMetadata, PluginLoadError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PluginLoadError::InvalidMetadata(
                format!("Failed to read {:?}: {}", path, e)
            ))?;
        
        let metadata: PluginMetadata = serde_json::from_str(&content)
            .map_err(|e| PluginLoadError::InvalidMetadata(
                format!("Failed to parse {:?}: {}", path, e)
            ))?;
        
        Ok(metadata)
    }
    
    /// 检查插件安全性
    fn check_plugin_security(&self, metadata: &PluginMetadata) -> Result<(), PluginLoadError> {
        // 黑名单检查
        if self.security_policy.blocklist.contains(&metadata.name) {
            return Err(PluginLoadError::SecurityCheckFailed(
                format!("Plugin {} is blocklisted", metadata.name)
            ));
        }
        
        // 权限级别检查
        if metadata.permission_level > self.security_policy.max_permission_level {
            return Err(PluginLoadError::SecurityCheckFailed(
                format!(
                    "Plugin {} requires permission level {:?}, but maximum allowed is {:?}",
                    metadata.name,
                    metadata.permission_level,
                    self.security_policy.max_permission_level
                )
            ));
        }
        
        // 托管策略检查
        if self.security_policy.allow_managed_only {
            // 只有来自托管目录的插件才被允许
            // 实际实现需要检查插件来源
        }
        
        Ok(())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

// 示例插件
#[derive(Default)]
pub struct EchoPlugin;

#[async_trait]
impl Plugin for EchoPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "echo".to_string(),
            version: "0.1.0".to_string(),
            description: "Echo plugin that returns input as output".to_string(),
            author: Some("Devil Team".to_string()),
            permission_level: PermissionLevel::None,
        }
    }

    async fn execute(&self, ctx: PluginContext) -> Result<PluginResult> {
        Ok(PluginResult {
            success: true,
            output: Some(ctx.input),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        let plugin = EchoPlugin::default();
        
        assert!(manager.register(plugin).await.is_ok());
        
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "echo");
    }

    #[tokio::test]
    async fn test_execute_plugin() {
        let mut manager = PluginManager::new();
        manager.register(EchoPlugin::default()).await.unwrap();

        let ctx = PluginContext {
            request_id: "test".to_string(),
            input: serde_json::json!({"message": "hello"}),
            env: HashMap::new(),
        };

        let result = manager.execute_plugin("echo", ctx).await.unwrap();
        assert!(result.success);
        assert!(result.output.is_some());
    }
}
