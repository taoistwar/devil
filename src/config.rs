//! 配置模块

pub struct Config {
    pub api_key: Option<String>,
    pub model: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            api_key: std::env::var("DEVIL_API_KEY").ok(),
            model: std::env::var("DEVIL_MODEL").unwrap_or_else(|_| "claude-3-sonnet".to_string()),
        }
    }
}
