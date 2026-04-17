//! CLI 交互模块

use anyhow::Result;

pub async fn run_once(prompt: &str) -> Result<()> {
    println!("Running: {}", prompt);
    println!("(单次执行功能待实现)");
    Ok(())
}

pub async fn run_repl() -> Result<()> {
    println!("Starting REPL mode...");
    println!("(交互模式待实现)");
    Ok(())
}

pub fn show_config() -> Result<()> {
    println!("Configuration:");
    println!("  (配置显示待实现)");
    Ok(())
}
