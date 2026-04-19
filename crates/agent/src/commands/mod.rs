//! Slash Commands Module
//!
//! 实现与 Claude Code 对齐的斜杠命令系统
//! 100+ 命令，每个命令一个独立的 Rust 模块

pub mod cmd_trait;
pub mod registry;

pub mod core;
pub mod config;
pub mod advanced;
pub mod edit;
pub mod collaboration;
pub mod system;

pub use cmd_trait::{SlashCommand, CommandResult, CommandContext};
pub use registry::CommandRegistry;

#[cfg(test)]
mod tests;
