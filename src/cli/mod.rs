//! CLI module - Command-line interface components
//!
//! Provides structured command dispatch following Claude Code's cli.tsx patterns.

pub mod commands;
pub mod dispatcher;
pub mod error;
pub mod init;

use std::io::{self, Write};
use anyhow::Result;
use devil_agent_core::commands::{CommandContext, global_registry};
use devil_agent_core::{Agent, AgentConfig, UserMessage, AgentMessage as Message};
pub use dispatcher::Dispatcher;
pub use error::CliError;

/// Application name
pub const APP_NAME: &str = "devil";

/// Version constant - injected at compile time via build.rs or env!
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the CLI application
#[allow(dead_code)]
pub async fn init() -> Result<()> {
    init::init().await
}

/// Run a single task with the agent
pub async fn run_once(prompt: &str) -> Result<()> {
    tracing::info!("Executing single task: {}", prompt);

    // Check for API key when running tasks
    let config = crate::config::Config::load().unwrap_or_default();
    if !config.has_api_key() {
        tracing::warn!("API key not configured. Set DEVIL_API_KEY to enable model calls.");
        println!("Warning: API key not configured. Set DEVIL_API_KEY to enable model calls.");
        println!("For testing, use: DEVIL_MOCK_MODEL=1");
    }

    // Create agent config
    let agent_config = AgentConfig {
        name: "devil-cli".to_string(),
        model: config.model.clone(),
        system_prompt: get_system_prompt(),
        max_turns: config.max_turns,
        max_context_tokens: config.max_context_tokens,
        ..Default::default()
    };

    // Create and initialize agent
    let agent = Agent::new(agent_config)?;
    agent.initialize().await?;

    // Create user message
    let user_message = Message::User(UserMessage::text(prompt));

    // Run agent
    let result = agent.run_once(user_message).await?;

    // Print result
    println!("\n--- Result ---");
    println!("Turns: {}", result.turn_count);
    println!("Terminal: {:?}", result.terminal.reason);
    
    // Print final message if any
    if let Some(last_msg) = result.messages.last() {
        match last_msg {
            Message::Assistant(asm) => {
                let text = asm.text_content();
                if !text.is_empty() {
                    println!("\nResponse:\n{}", text);
                }
            }
            _ => {}
        }
    }

    // Shutdown agent
    agent.shutdown().await?;

    Ok(())
}

fn get_system_prompt() -> String {
    r#"You are Devil Agent, an AI-powered development assistant built on MonkeyCode AI framework.

You are helpful, concise, and practical. You assist users with software engineering tasks including:
- Writing, reading, and modifying code
- Executing commands and debugging
- Searching and analyzing codebases
- Managing files and project structure

When working with files, prefer direct edits using the Write/Edit tools. Keep responses concise."#.to_string()
}

/// Run the interactive REPL
pub async fn run_repl() -> Result<()> {
    tracing::info!("Starting interactive REPL mode");
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  {} v{} - AI Development Assistant                       ║", APP_NAME, VERSION);
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Type /help for available commands, or enter a prompt to start.");
    println!("Press Ctrl+C or type /exit to quit.");
    println!();

    // Check for API key
    let config = crate::config::Config::load().unwrap_or_default();
    if !config.has_api_key() {
        println!("⚠️  Warning: API key not configured. Set DEVIL_API_KEY to enable model calls.");
        println!();
    }

    // Get command registry
    let registry = global_registry();
    let ctx = CommandContext::default();

    loop {
        print!("> ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Flush error: {}", e);
            break;
        }

        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle slash commands
        if input.starts_with('/') {
            let parts: Vec<&str> = input.split_whitespace().collect();
            let cmd_name = parts[0].trim_start_matches('/');
            let args: Vec<&str> = parts[1..].iter().copied().collect();

            match registry.execute(cmd_name, &ctx, &args).await {
                Some(result) => {
                    if let Some(output) = result.output {
                        println!("{}", output);
                    }
                    if !result.success {
                        if let Some(error) = result.error {
                            println!("❌ Error: {}", error);
                        }
                    }
                }
                None => {
                    println!("❌ Unknown command: /{}", cmd_name);
                    println!("Type /help for available commands.");
                }
            }
            println!();
            continue;
        }

        // Handle built-in commands
        match input {
            "/exit" | "/quit" | "/q" => {
                println!("Goodbye!");
                break;
            }
            "/help" | "/h" | "?" => {
                print_repl_help();
                continue;
            }
            _ => {
                // Regular prompt - would send to agent in full implementation
                tracing::info!("User prompt: {}", input);
                println!("📝 Processing: {}", input);
                println!("(Agent processing not yet connected - set DEVIL_API_KEY)");
                println!();
            }
        }
    }

    Ok(())
}

fn print_repl_help() {
    println!();
    println!("═══ Available Commands ═══");
    println!();
    println!("Slash Commands:");
    println!("  /help, /h, ?     Show this help message");
    println!("  /exit, /quit, /q Exit the REPL");
    println!();
    println!("Core Commands:");
    println!("  /compact         Manually compact context");
    println!("  /model [name]     Switch AI model");
    println!("  /clear           Clear conversation history");
    println!("  /plan            Enter plan mode");
    println!("  /review          Code review mode");
    println!();
    println!("Configuration:");
    println!("  /config          Show configuration");
    println!("  /theme [name]    Switch theme");
    println!("  /login           Login to service");
    println!("  /logout          Logout from service");
    println!();
    println!("Advanced:");
    println!("  /mcp             MCP server management");
    println!("  /skills          Skill management");
    println!("  /tasks           Task management");
    println!("  /memory          Memory management");
    println!("  /permissions     Permission settings");
    println!();
    println!("  ... and 85+ more commands");
    println!();
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
#[allow(dead_code)]
pub fn show_version() {
    println!("{} v{}", APP_NAME, VERSION);
}
