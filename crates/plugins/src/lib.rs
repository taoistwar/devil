//! Plugins crate - 提供插件系统和扩展机制
//! 
//! 本 crate 负责定义插件接口和插件管理功能

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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
    async fn initialize(&mut self) -> Result<()> {
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
    plugin: Box<dyn Plugin>,
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
                plugin: Box::new(plugin),
            },
        );

        info!("Registered plugin: {}", name);
        Ok(())
    }

    /// 获取插件
    pub async fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(name).map(|reg| &reg.plugin)
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
