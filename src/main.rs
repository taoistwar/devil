//! Devil Agent - 智能开发助手
//!
//! 基于 MonkeyCode AI 框架的命令行工具
//!
//! ## 功能特性
//!
//! - 🚀 流式对话交互
//! - 🔧 MCP 工具集成
//! - 📝 上下文智能管理
//! - 💰 成本追踪与预算控制
//!
//! ## 快速开始
//!
//! ```bash
//! # 安装
//! cargo install --path .
//!
//! # 运行
//! devil
//!
//! # 单次执行
//! devil run "帮我分析一下这个项目的结构"
//! ```

mod cli;
mod config;

use std::env;
use std::process::ExitCode;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const APP_NAME: &str = "devil";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    // Load .env file if present
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    // Parse global flags before command dispatch
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-vv");
    let log_level = if verbose { "debug" } else { "info" };

    // Fast path for --version: zero module loading
    // This matches cli.tsx lines 77-86 for instant response
    if args.len() == 2 {
        match args[1].as_str() {
            "--version" | "-v" | "-V" => {
                println!("{} v{}", APP_NAME, VERSION);
                return ExitCode::SUCCESS;
            }
            _ => {}
        }
    }

    // Fast path for --help
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }

    // Initialize logging for all other paths
    if let Err(e) = init_logging(log_level) {
        eprintln!("Failed to initialize logging: {}", e);
        return ExitCode::from(1);
    }

    tracing::debug!("Starting {} v{}", APP_NAME, VERSION);

    // Use dispatcher for all commands
    let dispatcher = cli::Dispatcher::new();

    // Filter out global flags before dispatching
    let cmd_args: Vec<String> = args[1..]
        .iter()
        .filter(|a| !a.starts_with("--verbose") && *a != "-vv")
        .cloned()
        .collect();

    match dispatcher.dispatch(&cmd_args) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            tracing::error!("Error: {}", e);
            cli::CliError::InitError(e.to_string()).exit_code()
        }
    }
}

fn init_logging(level: &str) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    // Production: JSON structured logging when DEVIL_LOG_JSON=true
    // Uses env_logger format for production, pretty format for development
    let is_production = env::var("DEVIL_LOG_JSON").unwrap_or_default() == "true";

    if is_production {
        // Production: structured logging to stdout
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(false)
            .with_level(true)
            .init();
    } else {
        // Development: human-readable logging
        tracing_subscriber::registry()
            .with(fmt::layer().with_target(true).with_thread_ids(false))
            .with(filter)
            .init();
    }

    Ok(())
}

fn print_help() {
    println!(
        "{} v{} - AI-Powered Development Assistant",
        APP_NAME, VERSION
    );
    println!();
    println!("Usage: {} <command> [arguments]", APP_NAME);
    println!();
    println!("Commands:");
    println!("  run <prompt>    Execute a single task");
    println!("  repl            Enter interactive mode");
    println!("  config          Show configuration");
    println!("  version, -v     Show version number");
    println!("  help, -h        Show this help message");
    println!();
    println!("Global Flags:");
    println!("  --verbose, -vv  Enable debug logging");
    println!("  --version, -v   Show version number");
    println!("  --help, -h      Show this help message");
    println!();
    println!("Environment Variables:");
    println!("  DEVIL_API_KEY              API key for authentication");
    println!("  DEVIL_MODEL                Model to use");
    println!("  DEVIL_PROVIDER             API provider (anthropic)");
    println!("  DEVIL_BASE_URL             API base URL (optional)");
    println!("  DEVIL_MAX_CONTEXT_TOKENS   Maximum context tokens");
    println!("  DEVIL_MAX_TURNS            Maximum turns per session");
    println!("  DEVIL_VERBOSE              Enable verbose logging");
    println!("  DEVIL_LOG_JSON             Enable JSON logging (true/false)");
    println!();
    println!("Examples:");
    println!("  {} run \"analyze project structure\"", APP_NAME);
    println!("  {} repl", APP_NAME);
    println!("  DEVIL_API_KEY=sk-xxx {} run \"hello\"", APP_NAME);
}
