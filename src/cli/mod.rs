//! CLI module - Command-line interface components
//!
//! Provides structured command dispatch following Claude Code's cli.tsx patterns.

pub mod commands;
pub mod dispatcher;
pub mod error;
pub mod init;

use anyhow::Result;

pub use dispatcher::Dispatcher;

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
    println!("Running task: {}", prompt);
    Ok(())
}

/// Run the interactive REPL
pub async fn run_repl() -> Result<()> {
    // Placeholder - will connect to agent core
    println!("Entering REPL mode...");
    println!("(REPL implementation pending)");
    Ok(())
}

/// Display configuration
pub fn show_config() -> Result<()> {
    let config = crate::config::Config::load().unwrap_or_default();

    println!("Configuration:");
    println!("  App: {}", config.app_name);
    println!("  Provider: {}", config.provider);
    println!("  Model: {}", config.model);
    println!(
        "  API Key: {}",
        if config.has_api_key() {
            "***"
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

    Ok(())
}

/// Display version
pub fn show_version() {
    println!("{} v{}", APP_NAME, VERSION);
}
