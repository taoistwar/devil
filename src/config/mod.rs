//! Configuration module
//!
//! Provides configuration management with support for:
//! - Config file loading (TOML)
//! - Environment variable overrides
//! - Runtime settings management

pub mod settings;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Main configuration struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application name
    pub app_name: String,
    /// API provider (e.g., "anthropic", "openai")
    pub provider: String,
    /// Model name
    pub model: String,
    /// API key (from config file or environment)
    pub api_key: Option<String>,
    /// Maximum context tokens
    pub max_context_tokens: usize,
    /// Maximum turns per session
    pub max_turns: usize,
    /// Tool execution timeout in seconds
    pub tool_timeout_secs: u64,
    /// Enable verbose logging
    pub verbose: bool,
    /// Custom settings
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: "devil".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key: None,
            max_context_tokens: 200000,
            max_turns: 50,
            tool_timeout_secs: 300,
            verbose: false,
            custom: HashMap::new(),
        }
    }
}

impl Config {
    /// Load configuration from file and environment
    ///
    /// Priority: ENV > config file > defaults
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        // Load from config file if exists
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                let file_config: ConfigFile = toml::from_str(&content)?;
                config.apply_file_config(file_config);
            }
        }

        // Override with environment variables
        config.apply_env_overrides();

        Ok(config)
    }

    /// Get config file path (~/.devil/config.toml)
    fn config_file_path() -> Option<PathBuf> {
        let home = env::var("HOME").ok()?;
        let path = PathBuf::from(home).join(".devil").join("config.toml");
        Some(path)
    }

    /// Apply environment variable overrides
    ///
    /// Environment variables with DEVIL_ prefix override config values:
    /// - DEVIL_API_KEY → api_key
    /// - DEVIL_MODEL → model
    /// - DEVIL_PROVIDER → provider
    /// - DEVIL_MAX_CONTEXT_TOKENS → max_context_tokens
    /// - DEVIL_MAX_TURNS → max_turns
    /// - DEVIL_VERBOSE → verbose
    fn apply_env_overrides(&mut self) {
        if let Ok(api_key) = env::var("DEVIL_API_KEY") {
            self.api_key = Some(api_key);
        }

        if let Ok(model) = env::var("DEVIL_MODEL") {
            self.model = model;
        }

        if let Ok(provider) = env::var("DEVIL_PROVIDER") {
            self.provider = provider;
        }

        if let Ok(max_tokens) = env::var("DEVIL_MAX_CONTEXT_TOKENS") {
            if let Ok(val) = max_tokens.parse() {
                self.max_context_tokens = val;
            }
        }

        if let Ok(max_turns) = env::var("DEVIL_MAX_TURNS") {
            if let Ok(val) = max_turns.parse() {
                self.max_turns = val;
            }
        }

        if let Ok(verbose) = env::var("DEVIL_VERBOSE") {
            self.verbose = verbose == "1" || verbose == "true";
        }
    }

    /// Apply configuration from file
    fn apply_file_config(&mut self, file_config: ConfigFile) {
        if let Some(app_name) = file_config.app_name {
            self.app_name = app_name;
        }
        if let Some(provider) = file_config.provider {
            self.provider = provider;
        }
        if let Some(model) = file_config.model {
            self.model = model;
        }
        if let Some(api_key) = file_config.api_key {
            self.api_key = Some(api_key);
        }
        if let Some(max_context_tokens) = file_config.max_context_tokens {
            self.max_context_tokens = max_context_tokens;
        }
        if let Some(max_turns) = file_config.max_turns {
            self.max_turns = max_turns;
        }
        if let Some(tool_timeout_secs) = file_config.tool_timeout_secs {
            self.tool_timeout_secs = tool_timeout_secs;
        }
        if let Some(verbose) = file_config.verbose {
            self.verbose = verbose;
        }
        if let Some(custom) = file_config.custom {
            self.custom = custom;
        }
    }

    /// Get a custom setting value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }

    /// Set a custom setting value
    pub fn set(&mut self, key: String, value: String) {
        self.custom.insert(key, value);
    }

    /// Check if API key is configured
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some() && !self.api_key.as_ref().unwrap().is_empty()
    }
}

/// Configuration file format (all fields optional)
#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(rename = "app.name")]
    app_name: Option<String>,
    #[serde(rename = "provider")]
    provider: Option<String>,
    #[serde(rename = "model")]
    model: Option<String>,
    #[serde(rename = "api_key")]
    api_key: Option<String>,
    #[serde(rename = "max_context_tokens")]
    max_context_tokens: Option<usize>,
    #[serde(rename = "max_turns")]
    max_turns: Option<usize>,
    #[serde(rename = "tool_timeout_secs")]
    tool_timeout_secs: Option<u64>,
    #[serde(rename = "verbose")]
    verbose: Option<bool>,
    #[serde(rename = "custom")]
    custom: Option<HashMap<String, String>>,
}

/// List of environment variables supported
pub fn list_env_vars() -> Vec<(&'static str, &'static str)> {
    vec![
        ("DEVIL_API_KEY", "API key for authentication"),
        (
            "DEVIL_MODEL",
            "Model to use (e.g., claude-sonnet-4-20250514)",
        ),
        ("DEVIL_PROVIDER", "API provider (anthropic, openai)"),
        (
            "DEVIL_MAX_CONTEXT_TOKENS",
            "Maximum context tokens (default: 200000)",
        ),
        ("DEVIL_MAX_TURNS", "Maximum turns per session (default: 50)"),
        ("DEVIL_VERBOSE", "Enable verbose logging (true/false)"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app_name, "devil");
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert!(!config.has_api_key());
    }

    #[test]
    fn test_config_env_override() {
        env::set_var("DEVIL_MODEL", "test-model");
        env::set_var("DEVIL_VERBOSE", "true");

        let config = Config::load().unwrap();
        assert_eq!(config.model, "test-model");
        assert!(config.verbose);

        env::remove_var("DEVIL_MODEL");
        env::remove_var("DEVIL_VERBOSE");
    }

    #[test]
    fn test_custom_settings() {
        let mut config = Config::default();
        config.set("theme".to_string(), "dark".to_string());
        assert_eq!(config.get("theme"), Some(&"dark".to_string()));
    }
}
