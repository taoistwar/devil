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

use anyhow::Result;
use std::env;
use std::process::ExitCode;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const APP_NAME: &str = "devil";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

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

    // Initialize logging for all other paths
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        return ExitCode::from(1);
    }

    // Use dispatcher for all commands
    let dispatcher = cli::Dispatcher::new();

    match dispatcher.dispatch(&args[1..]) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            tracing::error!("Error: {}", e);
            ExitCode::from(1)
        }
    }
}

fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    Ok(())
}
