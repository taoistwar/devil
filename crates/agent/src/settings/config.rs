//! 配置源定义模块
//! 
//! 定义六层配置源及其优先级

use serde::{Deserialize, Serialize};
use std::fmt;

/// 配置源类型
/// 
/// 按优先级从低到高排序
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub enum SettingSource {
    /// 插件设置（最低优先级，基础默认值）
    PluginSettings,
    /// 用户全局设置（`~/.claude/settings.json`）
    UserSettings,
    /// 项目共享设置（`.claude/settings.json`，入 Git）
    ProjectSettings,
    /// 项目本地设置（`.claude/settings.local.json`，不入 Git）
    LocalSettings,
    /// CLI 标志设置（`--settings` 参数，一次性覆盖）
    FlagSettings,
    /// 企业策略设置（最高优先级）
    PolicySettings,
}

impl SettingSource {
    /// 获取所有配置源，按优先级从低到高排序
    pub fn priority_order() -> Vec<SettingSource> {
        vec![
            SettingSource::PluginSettings,
            SettingSource::UserSettings,
            SettingSource::ProjectSettings,
            SettingSource::LocalSettings,
            SettingSource::FlagSettings,
            SettingSource::PolicySettings,
        ]
    }

    /// 获取可编辑的配置源（排除 policy 和 flag）
    pub fn editable_sources() -> Vec<SettingSource> {
        vec![
            SettingSource::PluginSettings,
            SettingSource::UserSettings,
            SettingSource::ProjectSettings,
            SettingSource::LocalSettings,
        ]
    }

    /// 获取可用于保存权限规则的配置源
    pub fn permission_save_sources() -> Vec<SettingSource> {
        vec![
            SettingSource::LocalSettings,
            SettingSource::ProjectSettings,
            SettingSource::UserSettings,
        ]
    }

    /// 判断此配置源是否支持持久化
    pub fn is_persistable(&self) -> bool {
        matches!(
            self,
            Self::UserSettings | Self::ProjectSettings | Self::LocalSettings
        )
    }

    /// 判断此配置源是否可信（用于安全敏感检查）
    /// 
    /// projectSettings 来自第三方仓库，不可信
    pub fn is_trusted(&self) -> bool {
        matches!(
            self,
            Self::UserSettings
                | Self::LocalSettings
                | Self::FlagSettings
                | Self::PolicySettings
        )
    }

    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PluginSettings => "plugin",
            Self::UserSettings => "user",
            Self::ProjectSettings => "project",
            Self::LocalSettings => "local",
            Self::FlagSettings => "flag",
            Self::PolicySettings => "policy",
        }
    }

    /// 获取简短显示名称（首字母大写）
    pub fn display_name_short(&self) -> &'static str {
        match self {
            Self::PluginSettings => "Plugin",
            Self::UserSettings => "User",
            Self::ProjectSettings => "Project",
            Self::LocalSettings => "Local",
            Self::FlagSettings => "Flag",
            Self::PolicySettings => "Policy",
        }
    }
}

impl fmt::Display for SettingSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// 配置源展示名称（全名，用于 UI）
pub fn get_setting_source_display_name(source: SettingSource) -> &'static str {
    match source {
        SettingSource::PluginSettings => "plugin",
        SettingSource::UserSettings => "user",
        SettingSource::ProjectSettings => "project",
        SettingSource::LocalSettings => "project, gitignored",
        SettingSource::FlagSettings => "cli flag",
        SettingSource::PolicySettings => "managed",
    }
}

/// 配置源详情描述
pub fn get_setting_source_description(source: SettingSource) -> &'static str {
    match source {
        SettingSource::PluginSettings => "插件提供的基础配置",
        SettingSource::UserSettings => "用户全局配置，个人默认值",
        SettingSource::ProjectSettings => "团队共享配置，提交到 Git",
        SettingSource::LocalSettings => "项目本地配置，不提交到 Git",
        SettingSource::FlagSettings => "CLI 参数覆盖，一次性生效",
        SettingSource::PolicySettings => "企业管理配置，最高优先级",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_order() {
        let order = SettingSource::priority_order();
        assert_eq!(order[0], SettingSource::PluginSettings);
        assert_eq!(order[order.len() - 1], SettingSource::PolicySettings);
    }

    #[test]
    fn test_is_trusted() {
        // projectSettings 不可信
        assert!(!SettingSource::ProjectSettings.is_trusted());
        
        // 其他来源可信
        assert!(SettingSource::UserSettings.is_trusted());
        assert!(SettingSource::LocalSettings.is_trusted());
        assert!(SettingSource::FlagSettings.is_trusted());
        assert!(SettingSource::PolicySettings.is_trusted());
    }

    #[test]
    fn test_is_persistable() {
        assert!(SettingSource::UserSettings.is_persistable());
        assert!(SettingSource::ProjectSettings.is_persistable());
        assert!(SettingSource::LocalSettings.is_persistable());
        
        assert!(!SettingSource::FlagSettings.is_persistable());
        assert!(!SettingSource::PolicySettings.is_persistable());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(SettingSource::UserSettings.display_name(), "user");
        assert_eq!(SettingSource::UserSettings.display_name_short(), "User");
        assert_eq!(
            get_setting_source_display_name(SettingSource::LocalSettings),
            "project, gitignored"
        );
    }
}
