//! Providers crate - 提供外部服务提供者实现
//!
//! 本 crate 负责实现各种外部服务的提供者（如 LLM、数据库、存储等）

pub mod anthropic;
pub mod openai;

pub use anthropic::{
    AnthropicClient, ChatMessage, ContentBlock, ContentBlockStart, ContentDelta, StreamEvent,
    ToolDef, Usage,
};
pub use openai::OpenAIClient;

use anyhow::Result;
use async_trait::async_trait;
use plugins::{Plugin, PluginContext, PluginMetadata, PluginResult};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// 提供者类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    /// LLM 提供者
    Llm { name: String },
    /// 数据库提供者
    Database { name: String },
    /// 存储服务提供者
    Storage { name: String },
    /// API 提供者
    Api { name: String },
}

/// 提供者配置 Trait
pub trait ProviderConfig: Send + Sync {
    /// 获取提供者名称
    fn name(&self) -> &str;
    /// 验证配置
    fn validate(&self) -> Result<()>;
}

/// 基础提供者 Trait
#[async_trait]
pub trait Provider: Send + Sync {
    /// 获取提供者类型
    fn provider_type(&self) -> ProviderType;
    /// 初始化提供者
    async fn initialize(&self) -> Result<()>;
    /// 执行提供者功能
    async fn call(&self, input: serde_json::Value) -> Result<serde_json::Value>;
    /// 关闭提供者
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

/// LLM 提供者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProviderConfig {
    pub name: String,
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
}

impl ProviderConfig for LlmProviderConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(anyhow::anyhow!("API key is required"));
        }
        if self.endpoint.is_empty() {
            return Err(anyhow::anyhow!("Endpoint is required"));
        }
        Ok(())
    }
}

/// LLM 提供者实现
pub struct LlmProvider {
    config: LlmProviderConfig,
}

impl LlmProvider {
    pub fn new(config: LlmProviderConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }
}

#[async_trait]
impl Provider for LlmProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Llm {
            name: self.config.name.clone(),
        }
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing LLM provider: {}", self.config.name);
        Ok(())
    }

    async fn call(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        debug!("LLM provider call with input: {:?}", input);
        // TODO: 实现实际的 LLM API 调用
        Ok(serde_json::json!({
            "response": "This is a placeholder response",
            "model": self.config.model
        }))
    }
}

/// 数据库提供者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseProviderConfig {
    pub name: String,
    pub connection_string: String,
    pub max_connections: u32,
}

impl ProviderConfig for DatabaseProviderConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.connection_string.is_empty() {
            return Err(anyhow::anyhow!("Connection string is required"));
        }
        Ok(())
    }
}

/// 数据库提供者实现
pub struct DatabaseProvider {
    config: DatabaseProviderConfig,
}

impl DatabaseProvider {
    pub fn new(config: DatabaseProviderConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }
}

#[async_trait]
impl Provider for DatabaseProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Database {
            name: self.config.name.clone(),
        }
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing database provider: {}", self.config.name);
        // TODO: 实现数据库连接池初始化
        Ok(())
    }

    async fn call(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        debug!("Database provider call with input: {:?}", input);
        // TODO: 实现实际的数据库操作
        Ok(serde_json::json!({
            "result": "placeholder"
        }))
    }
}

/// 提供者注册表
pub struct ProviderRegistry {
    providers: std::collections::HashMap<String, Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn Provider>) -> Result<()> {
        let name = match provider.provider_type() {
            ProviderType::Llm { ref name } => name.clone(),
            ProviderType::Database { ref name } => name.clone(),
            ProviderType::Storage { ref name } => name.clone(),
            ProviderType::Api { ref name } => name.clone(),
        };

        if self.providers.contains_key(&name) {
            return Err(anyhow::anyhow!("Provider already registered: {}", name));
        }

        self.providers.insert(name, provider);
        Ok(())
    }

    pub async fn initialize_all(&self) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.initialize().await?;
            info!("Initialized provider: {}", name);
        }
        Ok(())
    }

    pub async fn shutdown_all(&self) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.shutdown().await?;
            info!("Shutdown provider: {}", name);
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn Provider>> {
        self.providers.get(name)
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// 将 Plugin trait 适配器作为 Plugin 使用
#[async_trait]
impl Plugin for Box<dyn Provider> {
    fn metadata(&self) -> PluginMetadata {
        let provider_type = self.provider_type();
        let name = match &provider_type {
            ProviderType::Llm { name } => name.clone(),
            ProviderType::Database { name } => name.clone(),
            ProviderType::Storage { name } => name.clone(),
            ProviderType::Api { name } => name.clone(),
        };

        PluginMetadata {
            name,
            version: "0.1.0".to_string(),
            description: format!("Provider plugin for {:?}", provider_type),
            author: Some("Devil Team".to_string()),
            permission_level: Default::default(),
        }
    }

    async fn initialize(&self) -> Result<()> {
        self.as_ref().initialize().await
    }

    async fn execute(&self, ctx: PluginContext) -> Result<PluginResult> {
        let output = self.as_ref().call(ctx.input).await?;
        Ok(PluginResult {
            success: true,
            output: Some(output),
            error: None,
        })
    }

    async fn shutdown(&self) -> Result<()> {
        self.as_ref().shutdown().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_provider_config_validation() {
        let config = LlmProviderConfig {
            name: "test-llm".to_string(),
            api_key: "test-key".to_string(),
            endpoint: "https://api.example.com".to_string(),
            model: "gpt-4".to_string(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_provider_config_validation() {
        let config = DatabaseProviderConfig {
            name: "test-db".to_string(),
            connection_string: "sqlite::memory:".to_string(),
            max_connections: 10,
        };
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_provider_registry() {
        let mut registry = ProviderRegistry::new();

        let llm_config = LlmProviderConfig {
            name: "test-llm".to_string(),
            api_key: "test-key".to_string(),
            endpoint: "https://api.example.com".to_string(),
            model: "gpt-4".to_string(),
        };
        let llm_provider = LlmProvider::new(llm_config).unwrap();

        assert!(registry.register(Box::new(llm_provider)).is_ok());
        assert!(registry.get("test-llm").is_some());
    }
}
