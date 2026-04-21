//! 技能系统类型定义
//!
//! 定义 Skill 的核心类型、Frontmatter 字段和执行模式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill 命令定义
#[derive(Debug, Clone)]
pub struct SkillCommand {
    /// Skill 名称（显示名称，覆盖目录名）
    pub name: String,
    /// 描述
    pub description: String,
    /// 使用时机说明（AI 自动匹配依据）
    pub when_to_use: Option<String>,
    /// 工具白名单
    pub allowed_tools: Vec<String>,
    /// 参数提示
    pub argument_hint: Option<String>,
    /// 声明式参数名（用于 $ARGUMENTS 替换）
    pub arguments: Vec<String>,
    /// 模型覆盖
    pub model: Option<ModelOverride>,
    /// 努力级别
    pub effort: Option<EffortLevel>,
    /// 执行模式
    pub context: ExecutionContext,
    /// 指定 Agent 定义文件
    pub agent: Option<String>,
    /// 用户是否可 / 调用
    pub user_invocable: bool,
    /// 禁止 AI 自主调用
    pub disable_model_invocation: bool,
    /// 版本号
    pub version: Option<String>,
    /// 条件激活的文件路径模式
    pub paths: Vec<String>,
    /// Hook 配置
    pub hooks: HashMap<String, Vec<HookConfig>>,
    /// Shell 执行环境
    pub shell: Vec<String>,
    /// 来源
    pub source: SkillSource,
    /// 加载自
    pub loaded_from: SkillLoadSource,
    /// Skill 内容（Markdown）
    pub content: String,
    /// 所在目录的绝对路径
    pub skill_dir: String,
}

/// 模型覆盖
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelOverride {
    Opus,
    Sonnet,
    Haiku,
    Inherit,
    Custom(String),
}

/// 努力级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// 执行模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionContext {
    /// Inline 模式（默认）- 在主对话流中执行
    Inline,
    /// Fork 模式 - 在独立子 Agent 中执行
    Fork,
}

/// Skill 来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    /// 内置命令（硬编码）
    BuiltIn,
    /// Bundled Skills（编译时打包）
    Bundled,
    /// 磁盘 Skills（.claude/skills/）
    Disk,
    /// MCP Skills（动态发现）
    MCP,
    /// Legacy Commands（/commands/ 目录）
    Legacy,
}

/// 加载位置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkillLoadSource {
    /// 管理策略目录
    Managed,
    /// 用户全局目录
    UserSettings,
    /// 项目级目录
    ProjectSettings,
    /// 附加目录
    AddDir,
    /// MCP 服务器
    MCP,
}

/// Hook 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Hook 类型
    pub r#type: String,
    /// 命令
    pub command: Vec<String>,
}

/// Frontmatter 解析结果
#[derive(Debug, Clone, Deserialize)]
pub struct SkillFrontmatter {
    /// 显示名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 使用时机
    #[serde(rename = "when_to_use")]
    pub when_to_use: Option<String>,
    /// 工具白名单
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<Vec<String>>,
    /// 参数提示
    #[serde(rename = "argument-hint")]
    pub argument_hint: Option<String>,
    /// 声明式参数名
    pub arguments: Option<Vec<String>>,
    /// 模型覆盖
    pub model: Option<String>,
    /// 努力级别
    pub effort: Option<String>,
    /// 执行模式
    pub context: Option<String>,
    /// 指定 Agent
    pub agent: Option<String>,
    /// 用户可调用
    #[serde(rename = "user-invocable", default = "default_true")]
    pub user_invocable: bool,
    /// 禁用模型调用
    #[serde(rename = "disable-model-invocation", default)]
    pub disable_model_invocation: bool,
    /// 版本号
    pub version: Option<String>,
    /// 路径模式
    pub paths: Option<Vec<String>>,
    /// Hook 配置
    pub hooks: Option<HashMap<String, Vec<HookConfig>>>,
    /// Shell 环境
    pub shell: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}

impl SkillFrontmatter {
    /// 解析 frontmatter 字段到 SkillCommand
    pub fn to_skill_command(
        &self,
        name: String,
        content: String,
        skill_dir: String,
        source: SkillSource,
        loaded_from: SkillLoadSource,
    ) -> SkillCommand {
        SkillCommand {
            name: self.name.clone().unwrap_or(name),
            description: self.description.clone().unwrap_or_default(),
            when_to_use: self.when_to_use.clone(),
            allowed_tools: self.allowed_tools.clone().unwrap_or_default(),
            argument_hint: self.argument_hint.clone(),
            arguments: self.arguments.clone().unwrap_or_default(),
            model: self.model.as_ref().map(|m| match m.as_str() {
                "opus" => ModelOverride::Opus,
                "sonnet" => ModelOverride::Sonnet,
                "haiku" => ModelOverride::Haiku,
                "inherit" => ModelOverride::Inherit,
                _ => ModelOverride::Custom(m.clone()),
            }),
            effort: self
                .effort
                .as_ref()
                .map(|e| match e.to_lowercase().as_str() {
                    "low" => EffortLevel::Low,
                    "medium" => EffortLevel::Medium,
                    "high" => EffortLevel::High,
                    _ => EffortLevel::Medium,
                }),
            context: self
                .context
                .as_ref()
                .map(|c| match c.to_lowercase().as_str() {
                    "fork" => ExecutionContext::Fork,
                    _ => ExecutionContext::Inline,
                })
                .unwrap_or(ExecutionContext::Inline),
            agent: self.agent.clone(),
            user_invocable: self.user_invocable,
            disable_model_invocation: self.disable_model_invocation,
            version: self.version.clone(),
            paths: self.paths.clone().unwrap_or_default(),
            hooks: self.hooks.clone().unwrap_or_default(),
            shell: self.shell.clone().unwrap_or_default(),
            source,
            loaded_from,
            content,
            skill_dir,
        }
    }
}

/// 安全属性白名单（正向安全设计）
///
/// 任何不在白名单中的有意义的属性值都会触发权限请求
pub const SAFE_SKILL_PROPERTIES: &[&str] = &[
    "name",
    "description",
    "when_to_use",
    "allowed-tools",
    "argument-hint",
    "arguments",
    "model",
    "effort",
    "context",
    "agent",
    "user-invocable",
    "disable-model-invocation",
    "version",
    "paths",
    "hooks",
    "shell",
    // PromptCommand 属性
    "prompt",
    "category",
    "isInteractive",
    // CommandBase 属性
    "id",
    "source",
    "hidden",
];

/// Skill 使用统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillUsage {
    /// 使用次数
    pub usage_count: u32,
    /// 最后使用时间（时间戳）
    pub last_used: u64,
    /// 排名分数（指数衰减计算）
    pub ranking_score: f64,
}

impl SkillUsage {
    /// 计算排名分数（7 天半衰期）
    pub fn calculate_score(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let days_since_use = (now - self.last_used) as f64 / 86400.0;
        let decay_factor = (0.5_f64.powf(days_since_use / 7.0)).max(0.1);

        self.ranking_score = self.usage_count as f64 * decay_factor;
    }

    /// 记录使用（60 秒去抖）
    pub fn record_usage(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 60 秒去抖
        if now - self.last_used < 60 {
            return;
        }

        self.usage_count += 1;
        self.last_used = now;
        self.calculate_score();
    }
}

/// 条件激活的 Skill 存储
#[derive(Debug, Clone, Default)]
pub struct ConditionalSkills {
    /// 按路径模式存储的 Skill
    skills_by_pattern: HashMap<String, Vec<SkillCommand>>,
}

impl ConditionalSkills {
    /// 添加条件 Skill
    pub fn add(&mut self, pattern: impl Into<String>, skill: SkillCommand) {
        self.skills_by_pattern
            .entry(pattern.into())
            .or_default()
            .push(skill);
    }

    /// 检查并激活匹配路径的 Skill
    pub fn activate_for_path(&mut self, path: &str) -> Vec<SkillCommand> {
        let mut activated = Vec::new();

        self.skills_by_pattern.retain(|pattern, skills| {
            if Self::matches_pattern(pattern, path) {
                activated.append(skills);
                false // 移除已激活的
            } else {
                true // 保留
            }
        });

        activated
    }

    /// 检查路径是否匹配模式（gitignore 风格）
    fn matches_pattern(pattern: &str, path: &str) -> bool {
        // 简化实现：使用 glob 匹配
        // 实际应使用 ignore 库做 gitignore 风格匹配
        glob_match(pattern, path)
    }
}

/// Glob 模式匹配
fn glob_match(pattern: &str, path: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let (prefix, suffix) = (parts[0], parts[1]);
            return path.starts_with(prefix) && path.ends_with(suffix);
        }
    }
    pattern == path
}

/// 解析 YAML frontmatter
pub fn parse_frontmatter(content: &str) -> Result<SkillFrontmatter, String> {
    // 查找 frontmatter 边界
    let content = content.trim();
    if !content.starts_with("---") {
        return Err("Missing frontmatter start (---)".to_string());
    }

    let end = content[3..]
        .find("---")
        .map(|i| i + 3)
        .ok_or("Missing frontmatter end (---)")?;

    let yaml_content = &content[3..end];

    // 解析 YAML
    serde_yaml::from_str(yaml_content).map_err(|e| format!("Failed to parse frontmatter: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("src/**/*.ts", "src/auth/validate.ts"));
        assert!(glob_match("*.test.ts", "auth.test.ts"));
        assert!(!glob_match("src/**/*.ts", "test/auth.test.ts"));
    }

    #[test]
    fn test_skill_usage_score() {
        let mut usage = SkillUsage {
            usage_count: 10,
            last_used: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ranking_score: 0.0,
        };

        usage.calculate_score();
        assert_eq!(usage.ranking_score, 10.0); // 当天使用，无衰减

        // 模拟 7 天前使用
        usage.last_used -= 7 * 86400;
        usage.calculate_score();
        assert!((usage.ranking_score - 5.0).abs() < 0.1); // 半衰期
    }

    #[test]
    fn test_frontmatter_parsing() {
        let yaml_content = r#"---
name: Code Review
description: 代码审查技能
when_to_use: 需要审查代码时
allowed-tools:
  - Read
  - Write
effort: high
context: fork
---
这是技能内容"#;

        let result = parse_frontmatter(yaml_content);
        assert!(result.is_ok());

        let frontmatter = result.unwrap();
        assert_eq!(frontmatter.name, Some("Code Review".to_string()));
        assert_eq!(frontmatter.description, Some("代码审查技能".to_string()));
        assert_eq!(frontmatter.effort, Some("high".to_string()));
        assert_eq!(frontmatter.context, Some("fork".to_string()));
    }

    #[test]
    fn test_frontmatter_missing() {
        let content = "没有 frontmatter 的内容";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing frontmatter start"));
    }

    #[test]
    fn test_frontmatter_incomplete() {
        let content = "---\nname: test\n没有结束标记";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_override_parsing() {
        let yaml_content = r#"---
name: Test
model: opus
---
content"#;

        let result = parse_frontmatter(yaml_content);
        assert!(result.is_ok());

        let frontmatter = result.unwrap();
        assert_eq!(frontmatter.model, Some("opus".to_string()));
    }
}
