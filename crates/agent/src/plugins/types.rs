//! 插件系统类型定义
//!
//! 定义插件的核心类型、元数据和安全策略

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 插件元数据（从 package.json 或 plugin.json 提取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件名称（唯一的标识符）
    pub name: String,
    /// 显示名称
    pub display_name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 版本号
    pub version: String,
    /// 作者
    pub author: Option<String>,
    /// 许可证
    pub license: Option<String>,
    /// 仓库 URL
    pub repository: Option<String>,
    /// 主页 URL
    pub homepage: Option<String>,
    /// 最低 Claude Code 版本要求
    pub minimum_version: Option<String>,
    /// 插件入口点
    pub main: Option<String>,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 需要的权限
    pub required_permissions: Vec<String>,
    /// 配置 Schema
    pub config_schema: Option<serde_json::Value>,
    /// 贡献的 Skills
    pub contributed_skills: Vec<String>,
    /// 贡献的 Commands
    pub contributed_commands: Vec<String>,
}

/// 插件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Skills 插件（贡献 Skills）
    Skills,
    /// 工具插件（贡献 Tools）
    Tools,
    /// MCP 插件（MCP 服务器包装）
    MCP,
    /// 主题插件
    Theme,
    /// 扩展插件（通用）
    Extension,
}

/// 插件安装位置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginLocation {
    /// 全局插件目录（~/.claude/plugins/）
    Global,
    /// 项目级插件目录（.claude/plugins/）
    Project,
    /// 管理策略目录（$MANAGED_DIR/.claude/plugins/）
    Managed,
    /// 开发模式（符号链接）
    Development,
}

/// 插件安装状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    /// 已安装且激活
    Active,
    /// 已安装但禁用
    Disabled,
    /// 安装中
    Installing,
    /// 更新中
    Updating,
    /// 错误状态
    Error(String),
}

/// 已安装的插件
#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    /// 插件元数据
    pub metadata: PluginMetadata,
    /// 安装路径
    pub install_path: PathBuf,
    /// 安装位置
    pub location: PluginLocation,
    /// 安装时间
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// 状态
    pub status: PluginStatus,
    /// 配置值
    pub config: HashMap<String, serde_json::Value>,
}

/// 插件版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    /// 版本号
    pub version: String,
    /// 发布日期
    pub published_at: Option<String>,
    /// 下载地址
    pub download_url: Option<String>,
    /// 变更说明
    pub changelog: Option<String>,
    /// 是否为重大更新
    pub is_major: bool,
}

/// 插件更新策略
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginUpdatePolicy {
    /// 自动更新
    pub auto_update: bool,
    /// 忽略重大版本更新
    pub ignore_major_updates: bool,
    /// 忽略的插件列表
    pub ignored_plugins: Vec<String>,
}

/// 插件黑名单
#[derive(Debug, Clone, Default)]
pub struct PluginBlocklist {
    /// 被阻止的插件 ID 列表
    blocked_plugins: std::collections::HashSet<String>,
    /// 阻止原因
    block_reasons: HashMap<String, String>,
}

impl PluginBlocklist {
    /// 创建黑名单
    pub fn new() -> Self {
        Self {
            blocked_plugins: std::collections::HashSet::new(),
            block_reasons: HashMap::new(),
        }
    }

    /// 添加被阻止的插件
    pub fn block(&mut self, plugin_id: impl Into<String>, reason: impl Into<String>) {
        let plugin_id = plugin_id.into();
        self.blocked_plugins.insert(plugin_id.clone());
        self.block_reasons.insert(plugin_id, reason.into());
    }

    /// 检查插件是否被阻止
    pub fn is_blocked(&self, plugin_id: &str) -> bool {
        self.blocked_plugins.contains(plugin_id)
    }

    /// 获取阻止原因
    pub fn get_block_reason(&self, plugin_id: &str) -> Option<&str> {
        self.block_reasons.get(plugin_id).map(|s| s.as_str())
    }

    /// 从列表移除
    pub fn unblock(&mut self, plugin_id: &str) {
        self.blocked_plugins.remove(plugin_id);
        self.block_reasons.remove(plugin_id);
    }
}

/// 插件安全策略
#[derive(Debug, Clone)]
pub struct PluginSecurityPolicy {
    /// 仅允许托管插件
    pub allow_managed_only: bool,
    /// 需要签名验证
    pub require_signature: bool,
    /// 允许的发布者列表
    pub allowed_publishers: Vec<String>,
    /// 阻止的发布者列表
    pub blocked_publishers: Vec<String>,
    /// 最大权限级别
    pub max_permission_level: PermissionLevel,
}

impl Default for PluginSecurityPolicy {
    fn default() -> Self {
        Self {
            allow_managed_only: false,
            require_signature: true,
            allowed_publishers: Vec::new(),
            blocked_publishers: Vec::new(),
            max_permission_level: PermissionLevel::High,
        }
    }
}

/// 权限级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PermissionLevel {
    /// 无权限（仅读操作）
    None,
    /// 低权限（读 + 网络）
    Low,
    /// 中权限（读 + 写工作目录）
    Medium,
    /// 高权限（完全文件系统访问）
    High,
    /// 完全权限（包括执行任意代码）
    Full,
}

/// 插件配置存储
pub struct PluginConfigStorage {
    /// 配置目录
    config_dir: PathBuf,
}

impl PluginConfigStorage {
    /// 创建存储
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// 保存插件配置
    pub fn save_config(
        &self,
        plugin_id: &str,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        let config_file = self.config_dir.join(format!("{}.json", plugin_id));

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&config_file, json)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }

    /// 加载插件配置
    pub fn load_config(
        &self,
        plugin_id: &str,
    ) -> Result<HashMap<String, serde_json::Value>, String> {
        let config_file = self.config_dir.join(format!("{}.json", plugin_id));

        if !config_file.exists() {
            return Ok(HashMap::new());
        }

        let json = std::fs::read_to_string(&config_file)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&json).map_err(|e| format!("Failed to parse config: {}", e))
    }
}

/// 插件标识符工具
pub mod plugin_identifier {
    use super::*;

    /// 从路径提取插件 ID
    pub fn extract_plugin_id(path: &PathBuf) -> Option<String> {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    /// 验证插件 ID 格式
    pub fn is_valid_plugin_id(id: &str) -> bool {
        // 只允许字母数字、连字符、下划线
        id.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !id.is_empty()
            && id.len() <= 64
    }

    /// 规范化插件 ID
    pub fn normalize_plugin_id(id: &str) -> String {
        id.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_blocklist() {
        let mut blocklist = PluginBlocklist::new();

        blocklist.block("malicious-plugin", "Security vulnerability");

        assert!(blocklist.is_blocked("malicious-plugin"));
        assert_eq!(
            blocklist.get_block_reason("malicious-plugin"),
            Some("Security vulnerability")
        );

        blocklist.unblock("malicious-plugin");
        assert!(!blocklist.is_blocked("malicious-plugin"));
    }

    #[test]
    fn test_plugin_id_validation() {
        assert!(plugin_identifier::is_valid_plugin_id("my-plugin"));
        assert!(plugin_identifier::is_valid_plugin_id("my_plugin_123"));
        assert!(!plugin_identifier::is_valid_plugin_id("my plugin"));
        assert!(!plugin_identifier::is_valid_plugin_id(""));
        assert!(!plugin_identifier::is_valid_plugin_id(
            "a".repeat(65).as_str()
        ));
    }

    #[test]
    fn test_plugin_id_normalization() {
        assert_eq!(
            plugin_identifier::normalize_plugin_id("My-Plugin_123"),
            "my-plugin_123"
        );
    }
}
