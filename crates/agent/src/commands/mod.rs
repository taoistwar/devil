//! Slash Commands Module
//!
//! 实现与 Claude Code 对齐的斜杠命令系统
//! 100+ 命令，每个命令一个独立的 Rust 模块

pub mod cmd_trait;
pub mod registry;

pub mod advanced;
pub mod collaboration;
pub mod config;
pub mod core;
pub mod edit;
pub mod system;

pub use cmd_trait::{CommandContext, CommandResult, SlashCommand};
pub use registry::{global_registry, CommandRegistry};

#[cfg(test)]
mod tests;
