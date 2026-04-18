//! 工具系统模块
//!
//! 基于 Claude Code 架构实现，包含：
//! - Tool 五要素协议（名称、Schema、权限、执行、渲染）
//! - buildTool 工厂函数
//! - 并发分区策略
//! - StreamingToolExecutor 四阶段状态机
//! - 权限检查三层管线

pub mod bash_analyzer;
pub mod build_tool;
pub mod builtin;
pub mod executor;
pub mod partition;
pub mod registry;
pub mod tool;

pub use bash_analyzer::*;
pub use build_tool::*;
pub use builtin::*;
pub use executor::*;
pub use partition::*;
pub use registry::*;
pub use tool::*;
