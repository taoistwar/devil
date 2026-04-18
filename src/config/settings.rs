//! Runtime settings management
//!
//! Provides runtime configuration that can be modified during execution

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Runtime settings that can be modified during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSettings {
    /// Enable/disable tools
    pub tools_enabled: bool,
    /// Enable/disable streaming
    pub streaming_enabled: bool,
    /// Enable/disable color output
    pub no_color: bool,
    /// Output format (plain, json)
    pub output_format: OutputFormat,
    /// Log level
    pub log_level: LogLevel,
    /// Session ID (for tracking)
    pub session_id: Option<String>,
}

/// Output format for results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Plain,
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Plain
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl LogLevel {
    /// Convert to tracing level
    pub fn to_tracing_level(&self) -> &str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            tools_enabled: true,
            streaming_enabled: true,
            no_color: false,
            output_format: OutputFormat::Plain,
            log_level: LogLevel::Info,
            session_id: None,
        }
    }
}

/// Settings manager for runtime configuration
pub struct SettingsManager {
    settings: Arc<RwLock<RuntimeSettings>>,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new() -> Self {
        Self {
            settings: Arc::new(RwLock::new(RuntimeSettings::default())),
        }
    }

    /// Create with initial settings
    pub fn with_settings(settings: RuntimeSettings) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
        }
    }

    /// Get current settings (read-only)
    pub async fn get(&self) -> RuntimeSettings {
        self.settings.read().await.clone()
    }

    /// Update settings
    pub async fn update(&self, new_settings: RuntimeSettings) {
        let mut settings = self.settings.write().await;
        *settings = new_settings;
    }

    /// Enable/disable tools
    pub async fn set_tools_enabled(&self, enabled: bool) {
        let mut settings = self.settings.write().await;
        settings.tools_enabled = enabled;
    }

    /// Enable/disable streaming
    pub async fn set_streaming_enabled(&self, enabled: bool) {
        let mut settings = self.settings.write().await;
        settings.streaming_enabled = enabled;
    }

    /// Set output format
    pub async fn set_output_format(&self, format: OutputFormat) {
        let mut settings = self.settings.write().await;
        settings.output_format = format;
    }

    /// Set log level
    pub async fn set_log_level(&self, level: LogLevel) {
        let mut settings = self.settings.write().await;
        settings.log_level = level;
    }

    /// Set session ID
    pub async fn set_session_id(&self, session_id: Option<String>) {
        let mut settings = self.settings.write().await;
        settings.session_id = session_id;
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_settings_default() {
        let manager = SettingsManager::new();
        let settings = manager.get().await;
        assert!(settings.tools_enabled);
        assert!(settings.streaming_enabled);
        assert_eq!(settings.output_format, OutputFormat::Plain);
    }

    #[tokio::test]
    async fn test_update_settings() {
        let manager = SettingsManager::new();
        manager.set_tools_enabled(false).await;
        let settings = manager.get().await;
        assert!(!settings.tools_enabled);
    }
}
