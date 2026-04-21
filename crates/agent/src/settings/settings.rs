//! 设置加载与合并模块
//! 
//! 实现六层配置源的加载和深度合并逻辑

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use crate::settings::config::SettingSource;

/// 设置的 JSON 表示
pub type SettingsJson = serde_json::Value;

/// 设置验证错误
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    /// 错误来源文件
    pub file: String,
    /// 错误路径
    pub path: String,
    /// 错误消息
    pub message: String,
}

/// 带错误的设置结果
#[derive(Debug, Clone, Serialize)]
pub struct SettingsWithErrors {
    /// 合并后的设置
    pub settings: SettingsJson,
    /// 所有验证错误
    pub errors: Vec<ValidationError>,
}

/// 带来源的完整设置
#[derive(Debug, Clone, Serialize)]
pub struct SettingsWithSources {
    /// 合并后的有效设置
    pub effective: SettingsJson,
    /// 各来源的原始设置（按优先级排序）
    pub sources: Vec<SourceSettings>,
}

/// 单个来源的设置
#[derive(Debug, Clone, Serialize)]
pub struct SourceSettings {
    /// 配置来源
    pub source: SettingSource,
    /// 该来源的设置
    pub settings: SettingsJson,
}

/// 设置合并器
/// 
/// 负责合并多个来源的设置
pub struct SettingsMerger;

impl SettingsMerger {
    /// 合并多个来源的设置
    /// 
    /// # 合并规则
    /// 
    /// - **数组类型**：拼接并去重
    /// - **对象类型**：深度合并
    /// - **标量类型**：后者直接覆盖前者
    /// 
    /// # 参数
    /// 
    /// * `sources` - 按优先级从低到高排序的设置列表
    /// 
    /// # 返回
    /// 
    /// 合并后的设置
    pub fn merge(sources: &[(SettingSource, SettingsJson)]) -> SettingsJson {
        let mut merged = serde_json::json!({});

        for (source, settings) in sources {
            merged = Self::merge_with_customizer(merged, settings.clone());
        }

        merged
    }

    /// 使用自定义合并器合并两个设置对象
    fn merge_with_customizer(target: SettingsJson, source: SettingsJson) -> SettingsJson {
        match (target, source) {
            // 两个都是对象：深度合并
            (Value::Object(mut target_map), Value::Object(source_map)) => {
                for (key, source_value) in source_map {
                    let target_value = target_map.remove(&key);
                    
                    let merged_value = match target_value {
                        Some(target_val) => {
                            Self::merge_with_customizer(target_val, source_value)
                        }
                        None => source_value,
                    };
                    
                    target_map.insert(key, merged_value);
                }
                Value::Object(target_map)
            }
            // 两个都是数组：拼接并去重
            (Value::Array(mut target_arr), Value::Array(source_arr)) => {
                // 拼接数组
                target_arr.extend(source_arr);
                
                // 去重（仅对 JSON 值进行简单去重）
                let mut seen = std::collections::HashSet::new();
                let mut deduped = Vec::new();
                
                for item in target_arr {
                    let item_str = serde_json::to_string(&item).unwrap_or_default();
                    if seen.insert(item_str.clone()) {
                        deduped.push(item);
                    }
                }
                
                Value::Array(deduped)
            }
            // 其他类型：直接覆盖（使用 source 的值）
            (_, source_value) => source_value,
        }
    }
}

/// 设置加载器 Trait
/// 
/// 定义设置加载的接口
pub trait SettingsLoader {
    /// 从指定来源加载设置
    fn load_settings(&self, source: SettingSource) -> Option<SettingsJson>;
    
    /// 保存设置到指定来源
    fn save_settings(&self, source: SettingSource, settings: &SettingsJson) -> Result<(), String>;
}

/// 内存设置加载器（用于测试）
#[derive(Default)]
pub struct MemorySettingsLoader {
    settings: HashMap<SettingSource, SettingsJson>,
}

impl MemorySettingsLoader {
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    pub fn set(&mut self, source: SettingSource, settings: SettingsJson) {
        self.settings.insert(source, settings);
    }
}

impl SettingsLoader for MemorySettingsLoader {
    fn load_settings(&self, source: SettingSource) -> Option<SettingsJson> {
        self.settings.get(&source).cloned()
    }

    fn save_settings(&self, source: SettingSource, settings: &SettingsJson) -> Result<(), String> {
        // 简化实现：实际应该写入可变 HashMap
        Ok(())
    }
}

/// 文件系统设置加载器
pub struct FileSystemSettingsLoader {
    /// 项目根目录
    project_root: String,
    /// 用户配置目录
    user_config_dir: String,
}

impl FileSystemSettingsLoader {
    pub fn new(project_root: impl Into<String>, user_config_dir: impl Into<String>) -> Self {
        Self {
            project_root: project_root.into(),
            user_config_dir: user_config_dir.into(),
        }
    }

    /// 获取设置文件路径
    fn get_settings_path(&self, source: SettingSource) -> Option<String> {
        match source {
            SettingSource::UserSettings => {
                Some(format!("{}/.claude/settings.json", self.user_config_dir))
            }
            SettingSource::ProjectSettings => {
                Some(format!("{}/.claude/settings.json", self.project_root))
            }
            SettingSource::LocalSettings => {
                Some(format!("{}/.claude/settings.local.json", self.project_root))
            }
            SettingSource::FlagSettings => None, // CLI 参数指定
            SettingSource::PolicySettings => None, // 企业策略，特殊加载
            SettingSource::PluginSettings => None, // 插件提供
        }
    }

    /// 读取设置文件
    fn read_settings_file(&self, path: &str) -> Result<SettingsJson, String> {
        // 实际实现应该使用 std::fs::read_to_string
        // 这里简化处理
        Err("File read not implemented".to_string())
    }

    /// 写入设置文件
    fn write_settings_file(&self, path: &str, content: &str) -> Result<(), String> {
        // 实际实现应该使用 std::fs::write
        Err("File write not implemented".to_string())
    }
}

impl SettingsLoader for FileSystemSettingsLoader {
    fn load_settings(&self, source: SettingSource) -> Option<SettingsJson> {
        let path = self.get_settings_path(source)?;
        self.read_settings_file(&path).ok()
    }

    fn save_settings(&self, source: SettingSource, settings: &SettingsJson) -> Result<(), String> {
        let path = self.get_settings_path(source)
            .ok_or("Cannot save to this source")?;
        
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("JSON stringify error: {}", e))?;
        
        self.write_settings_file(&path, &content)
    }
}

/// 获取来自可信来源的特定设置值
/// 
/// **安全边界**：此函数系统性地排除 projectSettings
/// 防止来自第三方仓库的供应链攻击
/// 
/// # 参数
/// 
/// * `sources` - 各来源的设置
/// * `key_path` - 设置键的路径（支持嵌套，如 "permissions.defaultMode"）
/// 
/// # 返回
/// 
/// 第一个匹配的值
pub fn get_from_trusted_sources(
    sources: &[(SettingSource, SettingsJson)],
    key_path: &str,
) -> Option<SettingsJson> {
    // 按优先级从高到低检查可信来源
    let trusted_order = [
        SettingSource::PolicySettings,
        SettingSource::FlagSettings,
        SettingSource::LocalSettings,
        SettingSource::UserSettings,
    ];

    for &source in &trusted_order {
        if let Some((_, settings)) = sources.iter().find(|(s, _)| *s == source) {
            if let Some(value) = get_nested_value(settings, key_path) {
                return Some(value.clone());
            }
        }
    }

    None
}

/// 检查是否有任何可信来源设置了特定值
/// 
/// 用于权限相关的检查（如 skipDangerousModePermissionPrompt）
/// **projectSettings 被系统性排除**
pub fn has_trusted_source_setting(
    sources: &[(SettingSource, SettingsJson)],
    key_path: &str,
) -> bool {
    get_from_trusted_sources(sources, key_path).is_some()
}

/// 从嵌套的 JSON 对象中获取值
/// 
/// # 参数
/// 
/// * `json` - JSON 对象
/// * `path` - 点分隔的路径（如 "permissions.allow"）
/// 
/// # 返回
/// 
/// 如果路径存在则返回 Some(value)
fn get_nested_value(json: &SettingsJson, path: &str) -> Option<&SettingsJson> {
    let mut current = json;
    
    for key in path.split('.') {
        match current {
            Value::Object(map) => {
                current = map.get(key)?;
            }
            _ => return None,
        }
    }
    
    Some(current)
}

/// 设置变更检测器
/// 
/// 监控设置文件的变化
pub struct SettingsChangeDetector {
    /// 各来源的最后修改时间
    last_modified: HashMap<SettingSource, u64>,
}

impl SettingsChangeDetector {
    pub fn new() -> Self {
        Self {
            last_modified: HashMap::new(),
        }
    }

    /// 检查设置是否已变更
    pub fn has_changed(&self, source: SettingSource, current_time: u64) -> bool {
        self.last_modified
            .get(&source)
            .map_or(true, |&last| current_time > last)
    }

    /// 更新最后修改时间
    pub fn mark_checked(&mut self, source: SettingSource, timestamp: u64) {
        self.last_modified.insert(source, timestamp);
    }
}

impl Default for SettingsChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 应用设置变更
/// 
/// # 参数
/// 
/// * `current` - 当前设置
/// * `changes` - 变更内容
/// * `source` - 变更来源
/// 
/// # 返回
/// 
/// 更新后的设置
pub fn apply_settings_change(
    current: &SettingsJson,
    changes: &SettingsJson,
    _source: SettingSource,
) -> SettingsJson {
    SettingsMerger::merge_with_customizer(current.clone(), changes.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_scalar_override() {
        let sources = vec![
            (SettingSource::UserSettings, json!({"model": "sonnet"})),
            (SettingSource::LocalSettings, json!({"model": "opus"})),
        ];

        let merged = SettingsMerger::merge(&sources);
        assert_eq!(merged["model"], "opus");
    }

    #[test]
    fn test_merge_array_concat() {
        let sources = vec![
            (
                SettingSource::UserSettings,
                json!({
                    "permissions": {
                        "allow": ["Bash(ls)", "Bash(npm *)"]
                    }
                }),
            ),
            (
                SettingSource::ProjectSettings,
                json!({
                    "permissions": {
                        "allow": ["Read(*)", "Glob"]
                    }
                }),
            ),
        ];

        let merged = SettingsMerger::merge(&sources);
        let allow = &merged["permissions"]["allow"];
        
        assert!(allow.is_array());
        let allow_arr = allow.as_array().unwrap();
        
        // 数组应该包含所有项（拼接）
        assert!(allow_arr.iter().any(|v| v == "Bash(ls)"));
        assert!(allow_arr.iter().any(|v| v == "Bash(npm *)"));
        assert!(allow_arr.iter().any(|v| v == "Read(*)"));
        assert!(allow_arr.iter().any(|v| v == "Glob"));
    }

    #[test]
    fn test_merge_array_dedup() {
        let sources = vec![
            (
                SettingSource::UserSettings,
                json!({
                    "permissions": {
                        "allow": ["Bash(ls)", "Read"]
                    }
                }),
            ),
            (
                SettingSource::ProjectSettings,
                json!({
                    "permissions": {
                        "allow": ["Bash(ls)", "Glob"]
                    }
                }),
            ),
        ];

        let merged = SettingsMerger::merge(&sources);
        let allow = merged["permissions"]["allow"].as_array().unwrap();
        
        // "Bash(ls)" 应该只出现一次（去重）
        let count = allow.iter().filter(|v| v == &"Bash(ls)").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_merge_deep_object() {
        let sources = vec![
            (
                SettingSource::UserSettings,
                json!({
                    "permissions": {
                        "allow": ["Read"],
                        "defaultMode": "default"
                    },
                    "verbose": true
                }),
            ),
            (
                SettingSource::LocalSettings,
                json!({
                    "permissions": {
                        "deny": ["Bash(rm)"]
                    }
                }),
            ),
        ];

        let merged = SettingsMerger::merge(&sources);
        
        // permissions 应该深度合并
        assert_eq!(merged["permissions"]["allow"][0], "Read");
        assert_eq!(merged["permissions"]["deny"][0], "Bash(rm)");
        assert_eq!(merged["permissions"]["defaultMode"], "default");
        assert_eq!(merged["verbose"], true);
    }

    #[test]
    fn test_get_from_trusted_sources() {
        let sources = vec![
            (SettingSource::ProjectSettings, json!({"skipCheck": true})),
            (SettingSource::UserSettings, json!({"skipCheck": false})),
        ];

        // projectSettings 不可信，应该返回 userSettings 的值
        let value = get_from_trusted_sources(&sources, "skipCheck");
        assert_eq!(value, Some(json!(false)));
    }

    #[test]
    fn test_has_trusted_source_setting() {
        // projectSettings 被排除
        let sources = vec![(SettingSource::ProjectSettings, json!({"key": "value"}))];
        assert!(!has_trusted_source_setting(&sources, "key"));

        // userSettings 可信
        let sources = vec![(SettingSource::UserSettings, json!({"key": "value"}))];
        assert!(has_trusted_source_setting(&sources, "key"));
    }

    #[test]
    fn test_nested_value_extraction() {
        let json = json!({
            "permissions": {
                "allow": ["Read"],
                "defaultMode": "auto"
            }
        });

        assert_eq!(
            get_nested_value(&json, "permissions.defaultMode"),
            Some(&json!("auto"))
        );
        assert_eq!(
            get_nested_value(&json, "permissions.allow"),
            Some(&json!(["Read"]))
        );
        assert!(get_nested_value(&json, "nonexistent").is_none());
    }
}
