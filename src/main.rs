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

use anyhow::Result;
use std::env;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod cli;
mod config;

/// 版本号
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用名称
const APP_NAME: &str = "devil";

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging()?;

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: Missing prompt argument");
                eprintln!("Usage: {} run \"your prompt\"", APP_NAME);
                std::process::exit(1);
            }

            let prompt = args[2..].join(" ");
            cli::run_once(&prompt).await?;
        }
        "repl" => {
            cli::run_repl().await?;
        }
        "config" => {
            cli::show_config()?;
        }
        "version" | "-v" | "--version" => {
            println!("{} v{}", APP_NAME, VERSION);
        }
        "help" | "-h" | "--help" => {
            print_help();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}

/// 初始化日志系统
fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    Ok(())
}

/// 打印帮助信息
fn print_help() {
    println!(
        r#"{} v{}

Usage: {} <command> [arguments]

Commands:
  run <prompt>    执行单次任务
  repl            进入交互模式
  config          显示配置信息
  version, -v     显示版本号
  help, -h        显示此帮助信息

Examples:
  {} run "帮我分析一下这个项目的结构"
  {} repl
  {} config

Environment Variables:
  RUST_LOG        日志级别 (trace|debug|info|warn|error)
  DEVIL_CONFIG    配置文件路径
  DEVIL_API_KEY   API 密钥

For more information, visit: https://github.com/taoistwar/devil"#,
        APP_NAME,
        VERSION,
        APP_NAME,
        APP_NAME,
        APP_NAME,
        APP_NAME,
    );
}
