//! 权限管理模块
//!
//! 基于 Claude Code 四阶段权限检查流程实现：
//!
//! # 四阶段管线
//!
//! 1. **validateInput** - Zod Schema 验证：验证输入数据的合法性
//! 2. **hasPermissionsToUseTool** - 规则匹配：按 deny > ask > allow 优先级检查规则
//! 3. **checkPermissions** - 上下文评估：工具特定的上下文评估
//! 4. **交互式提示** - 用户确认：Hook/分类器/用户多决策者竞争
//!
//! # 权限模式谱系
//!
//! 从严格到宽松的五种模式：
//!
//! - `Default` - 逐次确认：每次工具调用都需要用户确认
//! - `Plan` - 只读为主：写入类工具被 deny，只读放行
//! - `Auto` - 自动审批：使用 AI 分类器代替人工审批
//! - `BypassPermissions` - 完全跳过：除 deny 规则外全部自动放行
//! - `Bubble` - 子智能体权限冒泡（内部模式）
//!
//! # 核心组件
//!
//! - [`context`] - 权限上下文和规则定义
//! - [`pipeline`] - 四阶段权限检查流程
//! - [`bash_analyzer`] - Bash 命令语义分析

pub mod bash_analyzer;
pub mod context;
pub mod pipeline;

pub use bash_analyzer::*;
pub use context::*;
pub use pipeline::*;
