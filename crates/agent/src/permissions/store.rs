//! 权限规则存储模块
//!
//! 实现规则存储（内存 + 文件持久化）：
//! - 支持加载/保存 TOML 配置
//! - 实现规则优先级排序
//! - 内存缓存 + 文件持久化

use crate::permissions::context::{
    apply_permission_update, apply_permission_updates, PermissionMode,
    PermissionRule, PermissionUpdate, RuleSource, ToolPermissionContext,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::sync::RwLock;

/// 规则存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStoreConfig {
    /// 规则文件路径
    pub rules_file: String,
    /// 是否自动保存
    pub auto_save: bool,
    /// 是否在启动时加载
    pub load_on_init: bool,
}

impl Default for RuleStoreConfig {
    fn default() -> Self {
        Self {
            rules_file: ".claude/permissions.toml".to_string(),
            auto_save: true,
            load_on_init: true,
        }
    }
}

/// 规则存储
///
/// 管理权限规则的内存缓存和文件持久化
pub struct RuleStore {
    /// 规则配置
    config: RuleStoreConfig,
    /// 内存中的规则上下文
    context: RwLock<ToolPermissionContext>,
    /// 规则文件路径
    rules_path: std::path::PathBuf,
}

impl RuleStore {
    /// 创建新的规则存储
    pub fn new(config: RuleStoreConfig) -> Self {
        let rules_path = Path::new(&config.rules_file).to_path_buf();
        Self {
            config,
            context: RwLock::new(ToolPermissionContext::with_defaults()),
            rules_path,
        }
    }

    /// 创建带默认配置的规则存储
    pub fn with_defaults() -> Self {
        Self::new(RuleStoreConfig::default())
    }

    /// 初始化存储（加载现有规则）
    pub async fn init(&self) -> Result<()> {
        if self.config.load_on_init {
            self.load().await?;
        }
        Ok(())
    }

    /// 获取当前权限上下文（只读）
    pub async fn get_context(&self) -> ToolPermissionContext {
        self.context.read().await.clone()
    }

    /// 获取可变权限上下文
    pub async fn get_mut_context(&self) -> tokio::sync::RwLockWriteGuard<'_, ToolPermissionContext> {
        self.context.write().await
    }

    /// 添加规则
    pub async fn add_rule(&self, rule: PermissionRule) -> Result<()> {
        {
            let mut ctx = self.context.write().await;
            ctx.add_rule(rule);
        }
        if self.config.auto_save {
            self.save().await?;
        }
        Ok(())
    }

    /// 添加允许规则
    pub async fn add_allow_rule(&self, source: RuleSource, target: impl Into<String>) -> Result<()> {
        let rule = PermissionRule::allow(source, target);
        self.add_rule(rule).await
    }

    /// 添加拒绝规则
    pub async fn add_deny_rule(&self, source: RuleSource, target: impl Into<String>) -> Result<()> {
        let rule = PermissionRule::deny(source, target);
        self.add_rule(rule).await
    }

    /// 添加询问规则
    pub async fn add_ask_rule(&self, source: RuleSource, target: impl Into<String>) -> Result<()> {
        let rule = PermissionRule::ask(source, target);
        self.add_rule(rule).await
    }

    /// 应用权限更新
    pub async fn apply_update(&self, update: &PermissionUpdate) -> Result<bool> {
        let ctx = self.context.read().await;
        let result = apply_permission_update(&ctx, update);
        drop(ctx);

        let has_persistable = result.has_persistable_update;
        let mut ctx = self.context.write().await;
        *ctx = result.new_context;

        if has_persistable && self.config.auto_save {
            drop(ctx);
            self.save().await?;
        }
        Ok(has_persistable)
    }

    /// 应用多个权限更新
    pub async fn apply_updates(&self, updates: &[PermissionUpdate]) -> Result<bool> {
        let ctx = self.context.read().await;
        let result = apply_permission_updates(&ctx, updates);
        drop(ctx);

        let has_persistable = result.has_persistable_update;
        let mut ctx = self.context.write().await;
        *ctx = result.new_context;

        if has_persistable && self.config.auto_save {
            drop(ctx);
            self.save().await?;
        }
        Ok(has_persistable)
    }

    /// 设置权限模式
    pub async fn set_mode(&self, mode: PermissionMode) -> Result<()> {
        let update = PermissionUpdate::set_mode(mode);
        self.apply_update(&update).await?;
        Ok(())
    }

    /// 从文件加载规则
    pub async fn load(&self) -> Result<()> {
        if !self.rules_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.rules_path)
            .context(format!("Failed to read rules from {:?}", self.rules_path))?;

        let stored: StoredPermissions = toml::from_str(&content)
            .context("Failed to parse rules file")?;

        let mut ctx = self.context.write().await;

        for rule in stored.allow_rules {
            ctx.add_rule(rule);
        }
        for rule in stored.deny_rules {
            ctx.add_rule(rule);
        }
        for rule in stored.ask_rules {
            ctx.add_rule(rule);
        }

        if let Some(mode) = stored.mode {
            ctx.mode = mode;
        }

        Ok(())
    }

    /// 保存规则到文件
    pub async fn save(&self) -> Result<()> {
        if let Some(parent) = self.rules_path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory {:?}", parent))?;
        }

        let ctx = self.context.read().await;

        let stored = StoredPermissions {
            allow_rules: ctx.allow_rules.values().flatten().cloned().collect(),
            deny_rules: ctx.deny_rules.values().flatten().cloned().collect(),
            ask_rules: ctx.ask_rules.values().flatten().cloned().collect(),
            mode: Some(ctx.mode),
        };

        let content = toml::to_string_pretty(&stored)
            .context("Failed to serialize rules")?;

        fs::write(&self.rules_path, content)
            .context(format!("Failed to write rules to {:?}", self.rules_path))?;

        Ok(())
    }

    /// 清除所有规则
    pub async fn clear(&self) -> Result<()> {
        {
            let mut ctx = self.context.write().await;
            ctx.allow_rules.clear();
            ctx.deny_rules.clear();
            ctx.ask_rules.clear();
        }
        if self.config.auto_save {
            self.save().await?;
        }
        Ok(())
    }
}

/// 存储的权限配置格式
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredPermissions {
    #[serde(rename = "allow", default)]
    allow_rules: Vec<PermissionRule>,
    #[serde(rename = "deny", default)]
    deny_rules: Vec<PermissionRule>,
    #[serde(rename = "ask", default)]
    ask_rules: Vec<PermissionRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<PermissionMode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::PermissionAction;

    #[tokio::test]
    async fn test_rule_store_add_rule() {
        let store = RuleStore::with_defaults();
        store.add_allow_rule(RuleSource::UserSettings, "Read").await.unwrap();

        let ctx = store.get_context().await;
        let rules = ctx.get_rules_by_priority(PermissionAction::Allow);
        assert_eq!(rules.len(), 1);
    }

    #[tokio::test]
    async fn test_rule_store_set_mode() {
        let store = RuleStore::with_defaults();
        store.set_mode(PermissionMode::BypassPermissions).await.unwrap();

        let ctx = store.get_context().await;
        assert_eq!(ctx.mode, PermissionMode::BypassPermissions);
    }
}
