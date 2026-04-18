pub mod brief;
pub mod ctx_inspect;

pub use ctx_inspect::{CtxInspectInput, CtxInspectOutput, CtxInspectTool};
pub use brief::BriefTool;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigGetInput {
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSetInput {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOutput {
    pub success: bool,
    pub key: Option<String>,
    pub value: Option<serde_json::Value>,
}

pub struct ConfigStore {
    config: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigStore {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let config = self.config.read().await;
        config.get(key).cloned()
    }

    pub async fn set(&self, key: &str, value: serde_json::Value) {
        let mut config = self.config.write().await;
        config.insert(key.to_string(), value);
    }

    pub async fn list(&self) -> HashMap<String, serde_json::Value> {
        let config = self.config.read().await;
        config.clone()
    }
}

pub struct ConfigGetTool {
    store: ConfigStore,
}

impl ConfigGetTool {
    pub fn new(store: ConfigStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for ConfigGetTool {
    type Input = ConfigGetInput;
    type Output = ConfigOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "config_get"
    }

    fn aliases(&self) -> &[&str] {
        &["get_config", "config"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Configuration key to retrieve"
                }
            }
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let key = input.key.clone();
        let value = if let Some(ref k) = key {
            self.store.get(k).await
        } else {
            Some(serde_json::json!(self.store.list().await))
        };

        let output = ConfigOutput {
            success: value.is_some(),
            key,
            value,
        };

        Ok(ToolResult::success("config_get-1", output))
    }
}

pub struct ConfigSetTool {
    store: ConfigStore,
}

impl ConfigSetTool {
    pub fn new(store: ConfigStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for ConfigSetTool {
    type Input = ConfigSetInput;
    type Output = ConfigOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "config_set"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Configuration key to set"
                },
                "value": {
                    "description": "Value to set"
                }
            },
            "required": ["key", "value"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        self.store.set(&input.key, input.value.clone()).await;

        let output = ConfigOutput {
            success: true,
            key: Some(input.key),
            value: Some(input.value),
        };

        Ok(ToolResult::success("config_set-1", output))
    }
}
