//! Skill 权限模型
//!
//! 实现五层权限检查和安全属性白名单

use crate::skills::types::{SkillCommand, SAFE_SKILL_PROPERTIES};
use serde::{Deserialize, Serialize};

/// 权限检查结果
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionCheckResult {
    /// 允许执行
    Allow,
    /// 拒绝执行
    Deny(String),
    /// 需要用户确认
    Ask {
        /// 原因
        reason: String,
        /// 建议的规则
        suggested_rules: Vec<String>,
    },
}

/// 权限检查器
pub struct SkillPermissionChecker {
    /// Allow 规则
    allow_rules: Vec<PermissionRule>,
    /// Deny 规则
    deny_rules: Vec<PermissionRule>,
    /// 是否启用远程 Skill 自动放行
    allow_remote_canonical: bool,
}

impl SkillPermissionChecker {
    /// 创建检查器
    pub fn new() -> Self {
        Self {
            allow_rules: Vec::new(),
            deny_rules: Vec::new(),
            allow_remote_canonical: false,
        }
    }

    /// 设置 Allow 规则
    pub fn with_allow_rules(mut self, rules: Vec<PermissionRule>) -> Self {
        self.allow_rules = rules;
        self
    }

    /// 设置 Deny 规则
    pub fn with_deny_rules(mut self, rules: Vec<PermissionRule>) -> Self {
        self.deny_rules = rules;
        self
    }

    /// 启用远程 Skill 自动放行
    pub fn with_remote_canonical_allow(mut self, enabled: bool) -> Self {
        self.allow_remote_canonical = enabled;
        self
    }

    /// 执行权限检查
    ///
    /// 五层权限检查：
    /// 1. Deny 规则匹配（支持精确匹配和 prefix:* 通配符）
    /// 2. 远程 canonical Skill 自动放行（需要 feature flag）
    /// 3. Allow 规则匹配
    /// 4. Safe Properties 白名单检查
    /// 5. Ask 用户确认
    pub fn check(&self, skill: &SkillCommand) -> PermissionCheckResult {
        // 第 1 层：Deny 规则匹配
        if let Some(reason) = self.check_deny_rules(skill) {
            return PermissionCheckResult::Deny(reason);
        }

        // 第 2 层：远程 canonical Skill 自动放行
        if self.allow_remote_canonical && self.is_remote_canonical(skill) {
            return PermissionCheckResult::Allow;
        }

        // 第 3 层：Allow 规则匹配
        if let Some(reason) = self.check_allow_rules(skill) {
            return PermissionCheckResult::Allow;
        }

        // 第 4 层：Safe Properties 白名单检查
        if self.skill_has_only_safe_properties(skill) {
            return PermissionCheckResult::Allow;
        }

        // 第 5 层：Ask 用户确认
        let suggested_rules = self.generate_suggested_rules(skill);
        PermissionCheckResult::Ask {
            reason: "Skill 包含需要权限的属性".to_string(),
            suggested_rules,
        }
    }

    /// 检查 Deny 规则
    fn check_deny_rules(&self, skill: &SkillCommand) -> Option<String> {
        for rule in &self.deny_rules {
            if rule.matches(skill) {
                return Some(format!("匹配 Deny 规则：{}", rule.pattern));
            }
        }
        None
    }

    /// 检查 Allow 规则
    fn check_allow_rules(&self, skill: &SkillCommand) -> Option<String> {
        for rule in &self.allow_rules {
            if rule.matches(skill) {
                return Some(format!("匹配 Allow 规则：{}", rule.pattern));
            }
        }
        None
    }

    /// 检查是否仅有安全属性
    ///
    /// Safe Properties 是一个包含 30 个属性名的白名单
    /// 任何不在白名单中的有意义的属性值都会触发权限请求
    fn skill_has_only_safe_properties(&self, skill: &SkillCommand) -> bool {
        // 获取所有有意义的属性值
        let meaningful_properties = self.get_meaningful_properties(skill);

        // 检查所有属性是否都在白名单中
        meaningful_properties
            .iter()
            .all(|prop| SAFE_SKILL_PROPERTIES.contains(&prop.as_str()))
    }

    /// 获取有意义的属性值
    ///
    /// 排除 undefined、null、空数组、空对象
    fn get_meaningful_properties(&self, skill: &SkillCommand) -> Vec<String> {
        let mut properties = Vec::new();

        // 检查各个字段是否有意义
        if !skill.allowed_tools.is_empty() {
            properties.push("allowed-tools".to_string());
        }
        if !skill.arguments.is_empty() {
            properties.push("arguments".to_string());
        }
        if skill.model.is_some() {
            properties.push("model".to_string());
        }
        if skill.effort.is_some() {
            properties.push("effort".to_string());
        }
        if skill.context != crate::skills::types::ExecutionContext::Inline {
            properties.push("context".to_string());
        }
        if skill.agent.is_some() {
            properties.push("agent".to_string());
        }
        if !skill.paths.is_empty() {
            properties.push("paths".to_string());
        }
        if !skill.hooks.is_empty() {
            properties.push("hooks".to_string());
        }
        if !skill.shell.is_empty() {
            properties.push("shell".to_string());
        }

        properties
    }

    /// 检查是否为远程 canonical Skill
    fn is_remote_canonical(&self, skill: &SkillCommand) -> bool {
        // 检查技能名称是否以 _canonical_ 开头
        skill.name.starts_with("_canonical_")
    }

    /// 生成建议的规则
    fn generate_suggested_rules(&self, skill: &SkillCommand) -> Vec<String> {
        let mut rules = Vec::new();

        // 精确匹配规则
        rules.push(format!("skill:{}", skill.name));

        // 前缀匹配规则
        if let Some(prefix) = skill.name.split('-').next() {
            rules.push(format!("skill:{}:*", prefix));
        }

        rules
    }
}

impl Default for SkillPermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 权限规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// 规则模式（支持精确匹配和 prefix:* 通配符）
    pub pattern: String,
    /// 规则类型
    pub rule_type: RuleType,
    /// 来源
    pub source: RuleSource,
}

impl PermissionRule {
    /// 创建规则
    pub fn new(pattern: impl Into<String>, rule_type: RuleType) -> Self {
        Self {
            pattern: pattern.into(),
            rule_type,
            source: RuleSource::User,
        }
    }

    /// 设置来源
    pub fn with_source(mut self, source: RuleSource) -> Self {
        self.source = source;
        self
    }

    /// 检查是否匹配 Skill
    pub fn matches(&self, skill: &SkillCommand) -> bool {
        match self.rule_type {
            RuleType::Skill => self.matches_skill(skill),
            RuleType::Tool => false, // TODO: 工具规则匹配
        }
    }

    /// 匹配 Skill 规则
    fn matches_skill(&self, skill: &SkillCommand) -> bool {
        // 精确匹配
        if self.pattern == format!("skill:{}", skill.name) {
            return true;
        }

        // 前缀匹配（prefix:*）
        if self.pattern.ends_with(":*") {
            let prefix = &self.pattern[..self.pattern.len() - 2];
            if let Some(stripped) = prefix.strip_prefix("skill:") {
                if skill.name.starts_with(stripped) {
                    return true;
                }
            }
        }

        false
    }
}

/// 规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    /// Skill 规则
    Skill,
    /// 工具规则
    Tool,
}

/// 规则来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSource {
    /// 用户设置
    User,
    /// 项目设置
    Project,
    /// 管理策略
    Managed,
    /// 自动添加
    Auto,
}

/// Shell 命令展开器
///
/// 展开复合命令并过滤危险命令
pub mod shell_expansion {
    use super::*;

    /// 展开复合命令
    ///
    /// 处理 &&、||、$() 等复合命令结构
    /// 返回展开后的简单命令列表
    pub fn expand_command(command: &str) -> Vec<String> {
        let mut commands = Vec::new();

        // 按 && 分割（顺序执行）
        for part in command.split("&&") {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                // 递归处理管道和重定向
                commands.extend(process_pipe_chain(trimmed));
            }
        }

        // 如果没有分割，直接处理
        if commands.is_empty() {
            commands.extend(process_pipe_chain(command));
        }

        commands
    }

    /// 处理管道链
    fn process_pipe_chain(command: &str) -> Vec<String> {
        let mut commands = Vec::new();

        // 按管道分割
        for part in command.split('|') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                // 处理命令替换 $()
                let expanded = expand_command_substitution(trimmed);
                commands.push(expanded);
            }
        }

        if commands.is_empty() {
            commands.push(command.to_string());
        }

        commands
    }

    /// 展开命令替换 $()
    fn expand_command_substitution(command: &str) -> String {
        // 简单实现：保留命令替换结构，但标记需要审查
        // 实际实现应该递归展开 $() 内的命令
        if command.contains("$(") {
            // 标记包含命令替换的命令
            format!("$SUBSTITUTION: {}", command)
        } else {
            command.to_string()
        }
    }

    /// 检查命令是否危险
    ///
    /// 过滤以下危险命令：
    /// - rm -rf /
    /// - curl | bash
    /// - wget | bash
    /// - 其他破坏性命令
    pub fn is_dangerous_command(command: &str) -> bool {
        let normalized = command.to_lowercase();

        // 危险模式列表
        let dangerous_patterns = [
            "rm -rf /",
            "rm -rf /home",
            "rm -rf /etc",
            "rm -rf /var",
            "rm -rf /*",
            "curl",
            "wget",
            "> /dev/sda",
            "dd if=",
            "mkfs",
            "fdisk",
            "chmod -R 777 /",
            "chown -R root:root /",
        ];

        // 检查是否包含危险模式
        for pattern in &dangerous_patterns {
            if normalized.contains(pattern) {
                return true;
            }
        }

        // 检查 curl | bash 模式
        if (normalized.contains("curl") || normalized.contains("wget"))
            && (normalized.contains("| bash") || normalized.contains("| sh"))
        {
            return true;
        }

        false
    }

    /// 检查并展开命令
    ///
    /// 返回展开后的命令列表和是否包含危险命令的标记
    pub fn check_and_expand(command: &str) -> (Vec<String>, bool) {
        let expanded = expand_command(command);
        let is_dangerous = expanded.iter().any(|cmd| is_dangerous_command(cmd));
        (expanded, is_dangerous)
    }
}

#[cfg(test)]
mod shell_expansion_tests {
    use super::shell_expansion::*;

    #[test]
    fn test_expand_simple_command() {
        let commands = expand_command("echo hello");
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "echo hello");
    }

    #[test]
    fn test_expand_and_chain() {
        let commands = expand_command("echo a && echo b && echo c");
        assert_eq!(commands.len(), 3);
        assert!(commands.contains(&"echo a".to_string()));
        assert!(commands.contains(&"echo b".to_string()));
        assert!(commands.contains(&"echo c".to_string()));
    }

    #[test]
    fn test_expand_pipe_chain() {
        let commands = expand_command("cat file | grep pattern | sort");
        assert_eq!(commands.len(), 3);
    }

    #[test]
    fn test_command_substitution() {
        let result = expand_command_substitution("echo $(whoami)");
        assert!(result.contains("$SUBSTITUTION"));
    }

    #[test]
    fn test_dangerous_rm_rf() {
        assert!(is_dangerous_command("rm -rf /"));
        assert!(is_dangerous_command("rm -rf /home"));
        assert!(!is_dangerous_command("rm -rf ./temp"));
    }

    #[test]
    fn test_dangerous_curl_bash() {
        assert!(is_dangerous_command("curl http://example.com | bash"));
        assert!(is_dangerous_command(
            "wget http://example.com/script.sh | sh"
        ));
        assert!(!is_dangerous_command("curl http://example.com"));
    }

    #[test]
    fn test_check_and_expand() {
        let (commands, dangerous) = check_and_expand("echo hello");
        assert!(!dangerous);
        assert_eq!(commands.len(), 1);

        let (commands, dangerous) = check_and_expand("rm -rf /");
        assert!(dangerous);

        let (commands, dangerous) = check_and_expand("echo a && rm -rf / && echo b");
        assert!(dangerous);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::{ExecutionContext, SkillLoadSource, SkillSource};
    use std::collections::HashMap;

    fn create_test_skill(name: &str) -> SkillCommand {
        SkillCommand {
            name: name.to_string(),
            description: "Test".to_string(),
            when_to_use: None,
            allowed_tools: vec![],
            argument_hint: None,
            arguments: vec![],
            model: None,
            effort: None,
            context: ExecutionContext::Inline,
            agent: None,
            user_invocable: true,
            disable_model_invocation: false,
            version: None,
            paths: vec![],
            hooks: HashMap::new(),
            shell: vec![],
            source: SkillSource::Disk,
            loaded_from: SkillLoadSource::ProjectSettings,
            content: "Test content".to_string(),
            skill_dir: "/test".to_string(),
        }
    }

    #[test]
    fn test_permission_deny_rule() {
        let checker = SkillPermissionChecker::new().with_deny_rules(vec![PermissionRule::new(
            "skill:dangerous-*",
            RuleType::Skill,
        )]);

        let skill = create_test_skill("dangerous-skill");
        let result = checker.check(&skill);

        assert!(matches!(result, PermissionCheckResult::Deny(_)));
    }

    #[test]
    fn test_permission_allow_rule() {
        let checker = SkillPermissionChecker::new()
            .with_allow_rules(vec![PermissionRule::new("skill:safe-*", RuleType::Skill)]);

        let skill = create_test_skill("safe-skill");
        let result = checker.check(&skill);

        assert!(matches!(result, PermissionCheckResult::Allow));
    }

    #[test]
    fn test_safe_properties() {
        let checker = SkillPermissionChecker::new();

        // 只有安全属性的 Skill
        let skill = create_test_skill("simple-skill");
        let result = checker.check(&skill);

        // 应该通过安全属性检查（没有非安全属性）
        assert!(matches!(result, PermissionCheckResult::Allow));
    }

    #[test]
    fn test_unsafe_properties() {
        let checker = SkillPermissionChecker::new();

        // 有非安全属性的 Skill（如 fork 上下文）
        let mut skill = create_test_skill("complex-skill");
        skill.context = ExecutionContext::Fork;
        skill.allowed_tools = vec!["Bash".to_string()];

        let result = checker.check(&skill);

        // 应该需要用户确认
        assert!(matches!(result, PermissionCheckResult::Ask { .. }));
    }

    #[test]
    fn test_remote_canonical() {
        let checker = SkillPermissionChecker::new().with_remote_canonical_allow(true);

        let skill = create_test_skill("_canonical_code-review");
        let result = checker.check(&skill);

        assert!(matches!(result, PermissionCheckResult::Allow));
    }

    #[test]
    fn test_rule_matching() {
        let rule = PermissionRule::new("skill:test-*", RuleType::Skill);
        let skill = create_test_skill("test-skill");

        assert!(rule.matches(&skill));

        let skill2 = create_test_skill("other-skill");
        assert!(!rule.matches(&skill2));
    }

    #[test]
    fn test_exact_match_rule() {
        let rule = PermissionRule::new("skill:exact-name", RuleType::Skill);
        let skill = create_test_skill("exact-name");

        assert!(rule.matches(&skill));

        let skill2 = create_test_skill("exact-name-other");
        assert!(!rule.matches(&skill2));
    }

    #[test]
    fn test_deny_priority_over_allow() {
        let checker = SkillPermissionChecker::new()
            .with_allow_rules(vec![PermissionRule::new("skill:test-*", RuleType::Skill)])
            .with_deny_rules(vec![PermissionRule::new(
                "skill:test-dangerous",
                RuleType::Skill,
            )]);

        let skill = create_test_skill("test-dangerous");
        let result = checker.check(&skill);

        // Deny 规则优先级更高
        assert!(matches!(result, PermissionCheckResult::Deny(_)));
    }

    #[test]
    fn test_suggested_rules() {
        let checker = SkillPermissionChecker::new();
        let skill = create_test_skill("code-review");

        if let PermissionCheckResult::Ask {
            suggested_rules, ..
        } = checker.check(&skill)
        {
            assert!(suggested_rules.contains(&"skill:code-review".to_string()));
            assert!(suggested_rules.contains(&"skill:code:*".to_string()));
        } else {
            panic!("Expected Ask result");
        }
    }

    #[test]
    fn test_meaningful_properties_detection() {
        let checker = SkillPermissionChecker::new();
        let mut skill = create_test_skill("test-skill");

        // 空技能应该没有有意义的属性
        let props = checker.get_meaningful_properties(&skill);
        assert!(props.is_empty());

        // 添加 allowed_tools 后应该有属性
        skill.allowed_tools = vec!["Bash".to_string()];
        let props = checker.get_meaningful_properties(&skill);
        assert!(props.contains(&"allowed-tools".to_string()));
    }
}
