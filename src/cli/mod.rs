//! CLI module - Command-line interface components
//!
//! Provides structured command dispatch following Claude Code's cli.tsx patterns.

pub mod commands;
pub mod dispatcher;
pub mod error;
pub mod init;

use anyhow::Result;
pub use dispatcher::Dispatcher;
pub use error::CliError;

/// Application name
pub const APP_NAME: &str = "devil";

/// Version constant - injected at compile time via build.rs or env!
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the CLI application
pub async fn init() -> Result<()> {
    init::init().await
}

/// Run a single task with the agent
pub async fn run_once(prompt: &str) -> Result<()> {
    // Placeholder - will connect to agent core
    tracing::info!("Executing single task: {}", prompt);
    println!("Running task: {}", prompt);

    // Check for API key when running tasks
    let config = crate::config::Config::load().unwrap_or_default();
    if !config.has_api_key() {
        tracing::warn!("API key not configured. Set DEVIL_API_KEY to enable model calls.");
    }

    Ok(())
}

/// Run the interactive REPL
pub async fn run_repl() -> Result<()> {
    // Placeholder - will connect to agent core
    tracing::info!("Starting interactive REPL mode");
    println!("Entering REPL mode...");
    println!("(REPL implementation pending)");

    // Check for API key when running REPL
    let config = crate::config::Config::load().unwrap_or_default();
    if !config.has_api_key() {
        tracing::warn!("API key not configured. Set DEVIL_API_KEY to enable model calls.");
    }

    Ok(())
}

/// Display configuration
pub fn show_config() -> anyhow::Result<()> {
    let config = crate::config::Config::load().map_err(|e| CliError::ConfigError(e.to_string()))?;

    println!("Configuration:");
    println!("  App: {}", config.app_name);
    println!("  Provider: {}", config.provider);
    println!("  Model: {}", config.model);
    println!(
        "  API Key: {}",
        if config.has_api_key() {
            "*** (configured)"
        } else {
            "not set"
        }
    );
    println!("  Max Context Tokens: {}", config.max_context_tokens);
    println!("  Max Turns: {}", config.max_turns);
    println!("  Tool Timeout: {}s", config.tool_timeout_secs);
    println!("  Verbose: {}", config.verbose);

    // Show environment variables
    println!("\nEnvironment Variables:");
    for (name, desc) in crate::config::list_env_vars() {
        println!("  {} - {}", name, desc);
    }

    // Show config file location hint
    println!("\nConfig file: ~/.devil/config.toml");

    Ok(())
}

/// Display version
pub fn show_version() {
    println!("{} v{}", APP_NAME, VERSION);
}
