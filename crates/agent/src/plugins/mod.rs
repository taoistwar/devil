//! 插件系统模块
//!
//! 实现插件的加载、安装、更新、安全策略和版本控制。
//!
//! ## 概述
//!
//! 插件系统允许扩展 Claude Code 的功能，支持 Skills、Tools、MCP 服务器、主题等插件类型。
//!
//! ## 插件来源
//!
//! | 来源 | 路径 | 说明 |
//! |------|------|------|
//! | **管理策略** | `$MANAGED_DIR/.claude/plugins/` | 企业managed 插件 |
//! | **全局目录** | `~/.claude/plugins/` | 用户安装的插件 |
//! | **项目级** | `.claude/plugins/` | 项目特定插件 |
//! | **开发模式** | 符号链接 | 开发中插件 |
//!
//! ## 核心功能
//!
//! ### 插件加载
//!
//! 加载协议：
//! 1. 扫描目录，识别 `package.json` 或 `plugin.json`
//! 2. 解析元数据（名称、版本、描述、权限等）
//! 3. 应用黑名单和安全策略过滤
//! 4. 加载到内存注册表
//!
//! ### 安全策略
//!
//! 三层防护：
//! 1. **黑名单**：阻止已知恶意插件
//! 2. **白名单模式**：`allow_managed_only` 仅允许托管插件
//! 3. **权限级别**：限制插件的能力（读/写/执行）
//!
//! ### 自动更新
//!
//! 更新策略：
//! - 自动检查更新（可配置间隔）
//! - 忽略重大版本更新（可选）
//! - 回滚机制（更新失败时）
//!
//! ### 版本控制
//!
//! 版本格式：SemVer 2.0.0
//! - 主版本号：不兼容的 API 变更
//! - 次版本号：向后兼容的功能新增
//! - 修订号：向后兼容的问题修正
//!
//! ## 模块结构
//!
//! ```
//! plugins/
//! ├── mod.rs              # 模块入口
//! ├── types.rs            # 类型定义
//! ├── loader.rs           # 插件加载器
//! ├── updater.rs          # 自动更新（TODO）
//! ├── policy.rs           # 安全策略（TODO）
//! └── registry.rs         # 插件注册表（TODO）
//! ```
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use agent::plugins::{PluginLoader, PluginBlocklist, PluginSecurityPolicy};
//! use std::path::PathBuf;
//!
//! // 创建插件加载器
//! let mut loader = PluginLoader::new()
//!     .with_plugin_dirs(vec![
//!         PathBuf::from("~/.claude/plugins"),
//!         PathBuf::from(".claude/plugins"),
//!     ])
//!     .with_blocklist(PluginBlocklist::new())
//!     .with_security_policy(PluginSecurityPolicy {
//!         allow_managed_only: false,
//!         ..Default::default()
//!     });
//!
//! // 加载所有插件
//! let count = loader.load_all().unwrap();
//! println!("Loaded {} plugins", count);
//!
//! // 获取已加载的插件
//! for plugin in loader.get_plugins() {
//!     println!("Plugin: {} v{}", plugin.metadata.name, plugin.metadata.version);
//! }
//! ```
//!
//! ## 与 Claude Code 对齐
//!
//! | Claude Code 文件 | 本实现 |
//! |-----------------|--------|
//! | `src/utils/plugins/pluginLoader.ts` | `loader.rs` |
//! | `src/utils/plugins/pluginIdentifier.ts` | `types::plugin_identifier` |
//! | `src/utils/plugins/pluginBlocklist.ts` | `types::PluginBlocklist` |
//! | `src/utils/plugins/pluginPolicy.ts` | `policy.rs`（TODO）|
//! | `src/utils/plugins/pluginAutoupdate.ts` | `updater.rs`（TODO）|
//! | `src/utils/plugins/pluginVersioning.ts` | `types::PluginVersion` |

pub mod loader;
pub mod types;

pub use loader::{PluginLoadError, PluginLoader};
pub use types::{
    plugin_identifier, InstalledPlugin, PermissionLevel, PluginBlocklist, PluginConfigStorage,
    PluginLocation, PluginMetadata, PluginSecurityPolicy, PluginStatus, PluginType,
    PluginUpdatePolicy, PluginVersion,
};
