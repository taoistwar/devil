//! 工具系统模块
//! 
//! 基于 Claude Code 架构实现，包含：
//! - Tool 五要素协议（名称、Schema、权限、执行、渲染）
//! - buildTool 工厂函数
//! - 并发分区策略
//! - StreamingToolExecutor 四阶段状态机
//! - 权限检查三层管线

pub mod tool;
pub mod build_tool;
pub mod registry;
pub mod executor;
pub mod partition;
pub mod builtin;
pub mod bash_analyzer;

pub use tool::*;
pub use build_tool::*;
pub use registry::*;
pub use executor::*;
pub use partition::*;
pub use builtin::*;
pub use bash_analyzer::*;
