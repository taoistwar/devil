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
    println!("Configuration:");
    println!("  App: {}", APP_NAME);
    println!("  Version: {}", VERSION);
    Ok(())
}

/// Display version
pub fn show_version() {
    println!("{} v{}", APP_NAME, VERSION);
}
