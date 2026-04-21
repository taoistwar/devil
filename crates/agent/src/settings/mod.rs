//! 设置与配置模块
//! 
//! 实现 Claude Code 的六层配置源体系：
//! 
//! # 六层配置源优先级（从低到高）
//! 
//! 1. **pluginSettings** - 插件设置（基础默认值）
//! 2. **userSettings** - 用户全局设置（`~/.claude/settings.json`）
//! 3. **projectSettings** - 项目共享设置（`.claude/settings.json`，入 Git）
//! 4. **localSettings** - 项目本地设置（`.claude/settings.local.json`，不入 Git）
//! 5. **flagSettings** - CLI 标志设置（`--settings` 参数，一次性覆盖）
//! 6. **policySettings** - 企业策略设置（最高优先级，企业级锁定）
//! 
//! # 合并规则
//! 
//! - **数组类型**：拼接并去重（而非替换）
//! - **对象类型**：深度合并，嵌套属性逐层覆盖
//! - **标量类型**：后者直接覆盖前者
//! 
//! # 安全边界
//! 
//! `projectSettings` 在所有安全敏感检查中被系统性排除，防止供应链攻击：
//! - skipDangerousModePermissionPrompt
//! - hasAutoModeOptIn
//! - getUseAutoModeDuringPlan
//! - allowManagedHooksOnly 检查
//! 
//! # 状态管理
//! 
//! 使用极简的 Store 模式（34 行实现）：
//! - getState() - 获取当前状态
//! - setState() - 通过 updater 函数更新状态
//! - subscribe() - 订阅状态变化

pub mod config;
pub mod settings;
pub mod store;
pub mod feature_flags;

pub use config::*;
pub use settings::*;
pub use store::*;
pub use feature_flags::*;
