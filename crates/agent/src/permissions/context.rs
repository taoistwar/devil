//! 权限上下文模块
//!
//! 实现权限检查所需的上下文信息：
//! - PermissionMode: 五种权限模式
//! - PermissionRule: 规则抽象
//! - ToolPermissionContext: 权限检查上下文
//! - PermissionDecision: 权限决策
//! - PermissionUpdate: 权限更新操作

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 权限模式
///
/// 定义了从严格到宽松的五种权限模式谱系
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionMode {
    /// default 模式：逐次确认
    /// 除了被明确 allow 规则放行的工具外，每次工具调用都需要用户确认
    Default,
    /// plan 模式：只读为主
    /// 将 Agent 限制在只读操作范围内，写入类工具被 deny
    Plan,
    /// auto 模式：自动审批（带分类器）
    /// 使用 AI 分类器来代替人工审批
    Auto,
    /// bypassPermissions 模式：完全跳过
    /// 除了被 deny 规则阻止和不可绕过的安全检查之外，所有工具调用都自动放行
    BypassPermissions,
    /// bubble 模式：子智能体权限冒泡（内部模式）
    /// 用于子智能体场景，权限检查会冒泡回主智能体
    #[serde(skip)]
    Bubble,
}

impl Default for PermissionMode {
    fn default() -> Self {
        Self::Default
    }
}

impl PermissionMode {
    /// 判断是否为内部模式（不对外暴露）
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Bubble)
    }

    /// 判断是否为外部可见模式
    pub fn is_external(&self) -> bool {
        !self.is_internal()
    }

    /// 判断此模式是否允许 bypass
    ///
    /// plan 模式在切换前如果是 bypass 模式，则 bypass 标志仍然可用
    pub fn allows_bypass(&self, bypass_available: bool) -> bool {
        matches!(self, Self::BypassPermissions) || (matches!(self, Self::Plan) && bypass_available)
    }
}

/// 权限规则来源
///
/// 七种规则来源按优先级排序（从高到低）：
/// 1. session - 会话级（最高优先级）
/// 2. command - 命令级
/// 3. cliArg - 命令行参数
/// 4. policySettings - 策略设置
/// 5. flagSettings - 功能标志
/// 6. localSettings - 本地设置（不提交 Git）
/// 7. projectSettings - 项目设置
/// 8. userSettings - 全局用户设置（最低优先级）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub enum RuleSource {
    /// 会话级规则（最高优先级）
    Session,
    /// 命令级规则
    Command,
    /// 命令行参数规则
    CliArg,
    /// 策略设置规则
    PolicySettings,
    /// 功能标志规则
    FlagSettings,
    /// 本地设置规则（不提交到版本控制）
    LocalSettings,
    /// 项目设置规则
    ProjectSettings,
    /// 全局用户设置规则（最低优先级）
    UserSettings,
}

impl RuleSource {
    /// 获取所有规则来源，按优先级从高到低排序
    pub fn priority_order() -> Vec<RuleSource> {
        vec![
            RuleSource::Session,
            RuleSource::Command,
            RuleSource::CliArg,
            RuleSource::PolicySettings,
            RuleSource::FlagSettings,
            RuleSource::LocalSettings,
            RuleSource::ProjectSettings,
            RuleSource::UserSettings,
        ]
    }

    /// 判断此来源是否支持持久化
    pub fn is_persistable(&self) -> bool {
        matches!(
            self,
            Self::LocalSettings | Self::UserSettings | Self::ProjectSettings
        )
    }
}

/// 权限行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    /// 允许执行
    Allow,
    /// 拒绝执行
    Deny,
    /// 询问用户
    Ask,
}

/// 权限规则
///
/// 将来源、行为和目标统一为一个结构化对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// 规则来源
    pub source: RuleSource,
    /// 权限行为
    pub action: PermissionAction,
    /// 规则目标（工具名或 MCP 服务器名）
    /// 支持精确匹配和通配符匹配
    /// 格式示例：
    /// - "Bash" - 精确匹配 Bash 工具
    /// - "Bash(npm test)" - 精确匹配 npm test 命令
    /// - "Bash(npm:*)" - 前缀匹配所有 npm 开头的命令
    /// - "Bash(git *)" - 通配符匹配 git 开头的命令
    /// - "Read" - 匹配所有 Read 工具调用
    /// - "mcp__server1__*" - MCP 服务器级通配符
    pub target: String,
}

impl PermissionRule {
    /// 创建新的权限规则
    pub fn new(source: RuleSource, action: PermissionAction, target: impl Into<String>) -> Self {
        Self {
            source,
            action,
            target: target.into(),
        }
    }

    /// 创建 allow 规则
    pub fn allow(source: RuleSource, target: impl Into<String>) -> Self {
        Self::new(source, PermissionAction::Allow, target)
    }

    /// 创建 deny 规则
    pub fn deny(source: RuleSource, target: impl Into<String>) -> Self {
        Self::new(source, PermissionAction::Deny, target)
    }

    /// 创建 ask 规则
    pub fn ask(source: RuleSource, target: impl Into<String>) -> Self {
        Self::new(source, PermissionAction::Ask, target)
    }

    /// 判断是否匹配给定的工具名
    ///
    /// 支持三种匹配模式：
    /// 1. 精确匹配：规则不含括号
    /// 2. 工具 + 精确命令：Bash(npm test)
    /// 3. 工具 + 前缀：Bash(npm:*)
    /// 4. 工具 + 通配符：Bash(git *)
    pub fn matches(&self, tool_name: &str, command: Option<&str>) -> bool {
        // 解析规则目标
        let (rule_tool, rule_content) = Self::parse_rule_target(&self.target);

        // 检查工具名是否匹配
        if rule_tool.to_lowercase() != tool_name.to_lowercase() {
            return false;
        }

        // 如果没有指定命令或规则没有内容，只匹配工具名
        let Some(cmd) = command else {
            return rule_content.is_none();
        };

        let Some(content) = rule_content else {
            return true;
        };

        // 检查内容匹配
        Self::matches_command(&content, cmd)
    }

    /// 解析规则目标
    ///
    /// 返回 (工具名，内容) 元组
    fn parse_rule_target(target: &str) -> (&str, Option<&str>) {
        // 检查是否包含括号
        if let Some(open_paren) = target.find('(') {
            if let Some(close_paren) = target.rfind(')') {
                if open_paren < close_paren {
                    let tool = &target[..open_paren];
                    let content = &target[open_paren + 1..close_paren];
                    return (tool, Some(content));
                }
            }
        }
        (target, None)
    }

    /// 判断命令是否匹配规则内容
    ///
    /// 支持三种匹配模式：
    /// 1. 精确匹配：内容完全相同
    /// 2. 前缀匹配：content 以 :* 结尾
    /// 3. 通配符匹配：content 包含 *（非尾部:*）
    fn matches_command(content: &str, command: &str) -> bool {
        // 检查前缀匹配语法 :*
        if content.ends_with(":*") {
            let prefix = &content[..content.len() - 2];
            return command.starts_with(prefix);
        }

        // 检查通配符匹配
        if content.contains('*') {
            return Self::matches_wildcard(content, command);
        }

        // 精确匹配
        content == command
    }

    /// 通配符匹配
    ///
    /// 将通配符 `*` 转换为正则表达式
    /// 支持转义 `\*` 和 `\\`
    /// 当模式以 ` *`（空格加通配符）结尾时，尾部空格和参数变为可选
    fn matches_wildcard(pattern: &str, text: &str) -> bool {
        // 构建正则表达式
        let mut regex_pattern = String::from("^");
        let mut chars = pattern.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    // 转义字符
                    if let Some(&next) = chars.peek() {
                        if next == '*' || next == '\\' {
                            regex_pattern.push(chars.next().unwrap());
                        } else {
                            regex_pattern.push('\\');
                            regex_pattern.push('\\');
                        }
                    }
                }
                '*' => {
                    // 通配符转换为 .*
                    regex_pattern.push_str(".*");
                }
                c => {
                    // 转义正则特殊字符
                    if "[(){}^$.|+".contains(c) {
                        regex_pattern.push('\\');
                    }
                    regex_pattern.push(c);
                }
            }
        }

        regex_pattern.push('$');

        // 特殊处理：如果模式以 ` *` 结尾且这是唯一的通配符
        // 使得尾部空格和参数变为可选
        if pattern.ends_with(" *") && pattern.matches('*').count() == 1 {
            let base_pattern = &pattern[..pattern.len() - 2];
            let optional_regex = format!("^{}( .*)?$", regex::escape(base_pattern));
            if let Ok(re) = Regex::new(&optional_regex) {
                return re.is_match(text);
            }
        }

        // 使用简单的通配符匹配（避免依赖 regex crate）
        Self::simple_wildcard_match(pattern, text)
    }

    /// 简单的通配符匹配实现
    fn simple_wildcard_match(pattern: &str, text: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('*').collect();

        if pattern_parts.len() == 1 {
            return pattern == text;
        }

        let mut text_pos = 0;
        for (i, part) in pattern_parts.iter().enumerate() {
            let part = part.trim_start_matches('\\');

            if i == 0 {
                // 第一部分必须匹配开头
                if !text.starts_with(part) {
                    return false;
                }
                text_pos = part.len();
            } else if i == pattern_parts.len() - 1 && !part.is_empty() {
                // 最后一部分必须匹配结尾
                if !text[text_pos..].ends_with(part) {
                    return false;
                }
            } else if !part.is_empty() {
                // 中间部分查找匹配
                if let Some(pos) = text[text_pos..].find(part) {
                    text_pos += pos + part.len();
                } else {
                    return false;
                }
            }
        }

        true
    }
}

/// 权限决策
///
/// 权限决策有三个来源：hook、user、classifier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior", rename_all = "lowercase")]
pub enum PermissionDecision {
    /// 允许执行
    Allow {
        /// 决策来源
        source: DecisionSource,
        /// 是否持久化（仅 user 来源有效）
        #[serde(default)]
        permanent: bool,
    },
    /// 询问用户
    Ask {
        /// 决策来源
        source: DecisionSource,
    },
    /// 拒绝执行
    Deny {
        /// 决策来源
        source: DecisionSource,
        /// 拒绝原因
        reason: String,
    },
}

/// 决策来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionSource {
    /// Hook 脚本决策（最高信任等级）
    Hook,
    /// 用户手动决策（中等信任等级）
    User,
    /// AI 分类器自动决策（最低信任等级）
    Classifier,
}

impl PermissionDecision {
    /// 创建 allow 决策
    pub fn allow(source: DecisionSource, permanent: bool) -> Self {
        Self::Allow { source, permanent }
    }

    /// 创建 ask 决策
    pub fn ask(source: DecisionSource) -> Self {
        Self::Ask { source }
    }

    /// 创建 deny 决策
    pub fn deny(source: DecisionSource, reason: impl Into<String>) -> Self {
        Self::Deny {
            source,
            reason: reason.into(),
        }
    }

    /// 获取决策来源
    pub fn source(&self) -> DecisionSource {
        match self {
            Self::Allow { source, .. } => *source,
            Self::Ask { source } => *source,
            Self::Deny { source, .. } => *source,
        }
    }

    /// 判断是否为永久决策
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::Allow {
                permanent: true,
                ..
            }
        )
    }
}

/// 权限上下文
///
/// 携带权限检查所需的所有上下文信息
/// 所有字段均为 readonly（不可变），确保并发安全
#[derive(Debug, Clone, Default)]
pub struct ToolPermissionContext {
    /// 当前权限模式
    pub mode: PermissionMode,
    /// 额外工作目录（允许访问的目录列表）
    pub extra_work_dirs: Vec<String>,
    /// allow 规则列表（按来源索引）
    pub allow_rules: HashMap<RuleSource, Vec<PermissionRule>>,
    /// deny 规则列表（按来源索引）
    pub deny_rules: HashMap<RuleSource, Vec<PermissionRule>>,
    /// ask 规则列表（按来源索引）
    pub ask_rules: HashMap<RuleSource, Vec<PermissionRule>>,
    /// bypass 模式可用性标志
    /// 为 true 表示用户原本使用 bypass 模式然后切换到其他模式
    pub bypass_available: bool,
    /// 是否应避免权限提示
    /// 用于非交互式会话
    pub avoid_permission_prompts: bool,
    /// 安全工具白名单（跳过分类器检查）
    pub safe_tools: Vec<String>,
    /// acceptEdits 模式标志（auto 模式优化）
    pub accept_edits_mode: bool,
}

impl ToolPermissionContext {
    /// 创建新的权限上下文
    pub fn new(mode: PermissionMode) -> Self {
        Self {
            mode,
            extra_work_dirs: Vec::new(),
            allow_rules: HashMap::new(),
            deny_rules: HashMap::new(),
            ask_rules: HashMap::new(),
            bypass_available: false,
            avoid_permission_prompts: false,
            safe_tools: Vec::new(),
            accept_edits_mode: false,
        }
    }

    /// 创建默认的权限上下文
    pub fn with_defaults() -> Self {
        let mut ctx = Self::new(PermissionMode::Default);
        // 默认安全工具列表
        ctx.safe_tools = vec![
            "Read".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
            "TodoWrite".to_string(),
        ];
        ctx
    }

    /// 添加规则到指定来源
    pub fn add_rule(&mut self, rule: PermissionRule) {
        let rules = match rule.action {
            PermissionAction::Allow => self.allow_rules.entry(rule.source).or_insert_with(Vec::new),
            PermissionAction::Deny => self.deny_rules.entry(rule.source).or_insert_with(Vec::new),
            PermissionAction::Ask => self.ask_rules.entry(rule.source).or_insert_with(Vec::new),
        };
        rules.push(rule);
    }

    /// 添加允许规则
    pub fn add_allow_rule(&mut self, source: RuleSource, target: impl Into<String>) {
        self.add_rule(PermissionRule::allow(source, target));
    }

    /// 添加拒绝规则
    pub fn add_deny_rule(&mut self, source: RuleSource, target: impl Into<String>) {
        self.add_rule(PermissionRule::deny(source, target));
    }

    /// 添加询问规则
    pub fn add_ask_rule(&mut self, source: RuleSource, target: impl Into<String>) {
        self.add_rule(PermissionRule::ask(source, target));
    }

    /// 获取所有规则的优先级排序列表
    ///
    /// 按优先级从高到低排序，用于规则匹配
    pub fn get_rules_by_priority(&self, action: PermissionAction) -> Vec<&PermissionRule> {
        let rules = match action {
            PermissionAction::Allow => &self.allow_rules,
            PermissionAction::Deny => &self.deny_rules,
            PermissionAction::Ask => &self.ask_rules,
        };

        let mut result = Vec::new();
        for source in RuleSource::priority_order() {
            if let Some(rules) = rules.get(&source) {
                for rule in rules {
                    result.push(rule);
                }
            }
        }
        result
    }

    /// 判断当前模式是否允许 bypass
    pub fn allows_bypass(&self) -> bool {
        self.mode.allows_bypass(self.bypass_available)
    }

    /// 判断工具是否在安全白名单中
    pub fn is_safe_tool(&self, tool_name: &str) -> bool {
        self.safe_tools.iter().any(|t| t == tool_name)
    }

    /// 添加工作目录
    pub fn add_work_dir(&mut self, dir: impl Into<String>) {
        self.extra_work_dirs.push(dir.into());
    }

    /// 判断目录是否在允许的工作目录列表中
    pub fn is_allowed_work_dir(&self, dir: &str) -> bool {
        self.extra_work_dirs.iter().any(|d| dir.starts_with(d))
    }

    /// 设置为 bypass 可用
    pub fn set_bypass_available(&mut self, available: bool) {
        self.bypass_available = available;
    }

    /// 设置避免权限提示标志
    pub fn set_avoid_permission_prompts(&mut self, avoid: bool) {
        self.avoid_permission_prompts = avoid;
    }

    /// 设置 acceptEdits 模式
    pub fn set_accept_edits_mode(&mut self, enabled: bool) {
        self.accept_edits_mode = enabled;
    }
}

/// 权限更新操作
///
/// 支持六种操作 × 五种配置源 = 30 种可能的更新
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PermissionUpdate {
    /// 添加规则
    AddRules {
        /// 规则来源
        source: RuleSource,
        /// 要添加的规则列表
        rules: Vec<PermissionRule>,
    },
    /// 替换规则
    ReplaceRules {
        /// 规则来源
        source: RuleSource,
        /// 新的规则列表
        rules: Vec<PermissionRule>,
    },
    /// 移除规则
    RemoveRules {
        /// 规则来源
        source: RuleSource,
        /// 要移除的规则目标列表
        targets: Vec<String>,
    },
    /// 设置权限模式
    SetMode {
        /// 新的权限模式
        mode: PermissionMode,
        /// 是否同时设置 bypass_available
        #[serde(default)]
        set_bypass_available: bool,
    },
    /// 添加工作目录
    AddWorkDir {
        /// 目录路径
        dir: String,
    },
    /// 移除工作目录
    RemoveWorkDir {
        /// 目录路径
        dir: String,
    },
}

impl PermissionUpdate {
    /// 创建添加规则的更新
    pub fn add_rules(source: RuleSource, rules: Vec<PermissionRule>) -> Self {
        Self::AddRules { source, rules }
    }

    /// 创建设置模式的更新
    pub fn set_mode(mode: PermissionMode) -> Self {
        Self::SetMode {
            mode,
            set_bypass_available: false,
        }
    }

    /// 判断此更新是否支持持久化
    pub fn is_persistable(&self) -> bool {
        match self {
            Self::AddRules { source, .. }
            | Self::ReplaceRules { source, .. }
            | Self::RemoveRules { source, .. } => source.is_persistable(),
            Self::SetMode { .. } => false,
            Self::AddWorkDir { .. } | Self::RemoveWorkDir { .. } => true,
        }
    }
}

/// 应用权限更新后的结果
#[derive(Debug, Clone)]
pub struct ApplyUpdateResult {
    /// 更新后的权限上下文
    pub new_context: ToolPermissionContext,
    /// 是否有持久化更新发生
    pub has_persistable_update: bool,
}

/// 应用单个权限更新到上下文
///
/// 返回更新后的新上下文（不可变更新模式）
pub fn apply_permission_update(
    context: &ToolPermissionContext,
    update: &PermissionUpdate,
) -> ApplyUpdateResult {
    let mut new_context = context.clone();
    let mut has_persistable_update = false;

    match update {
        PermissionUpdate::AddRules { source, rules } => {
            for rule in rules {
                let mut rule = rule.clone();
                rule.source = *source;
                new_context.add_rule(rule);
            }
            has_persistable_update = source.is_persistable();
        }
        PermissionUpdate::ReplaceRules { source, rules } => {
            // 清空指定来源的所有规则
            new_context.allow_rules.remove(source);
            new_context.deny_rules.remove(source);
            new_context.ask_rules.remove(source);
            // 添加新规则
            for rule in rules {
                let mut rule = rule.clone();
                rule.source = *source;
                new_context.add_rule(rule);
            }
            has_persistable_update = source.is_persistable();
        }
        PermissionUpdate::RemoveRules { source, targets } => {
            // 从指定来源移除匹配的规则
            for rules in [
                &mut new_context.allow_rules,
                &mut new_context.deny_rules,
                &mut new_context.ask_rules,
            ] {
                if let Some(rules) = rules.get_mut(source) {
                    rules.retain(|rule| !targets.contains(&rule.target));
                }
            }
            has_persistable_update = source.is_persistable();
        }
        PermissionUpdate::SetMode {
            mode,
            set_bypass_available,
        } => {
            // 保存旧的 bypass 状态
            let old_bypass = new_context.bypass_available;
            new_context.mode = *mode;
            if *set_bypass_available {
                new_context.bypass_available =
                    old_bypass || matches!(mode, PermissionMode::BypassPermissions);
            }
        }
        PermissionUpdate::AddWorkDir { dir } => {
            new_context.add_work_dir(dir.clone());
            has_persistable_update = true;
        }
        PermissionUpdate::RemoveWorkDir { dir } => {
            new_context.extra_work_dirs.retain(|d| d != dir);
            has_persistable_update = true;
        }
    }

    ApplyUpdateResult {
        new_context,
        has_persistable_update,
    }
}

/// 应用多个权限更新
pub fn apply_permission_updates(
    context: &ToolPermissionContext,
    updates: &[PermissionUpdate],
) -> ApplyUpdateResult {
    let mut result = ApplyUpdateResult {
        new_context: context.clone(),
        has_persistable_update: false,
    };

    for update in updates {
        let update_result = apply_permission_update(&result.new_context, update);
        result.new_context = update_result.new_context;
        result.has_persistable_update |= update_result.has_persistable_update;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_mode() {
        assert!(PermissionMode::Default.is_external());
        assert!(PermissionMode::Bubble.is_internal());
        assert!(!PermissionMode::Default.allows_bypass(false));
        assert!(PermissionMode::BypassPermissions.allows_bypass(false));
        assert!(PermissionMode::Plan.allows_bypass(true));
    }

    #[test]
    fn test_rule_source_priority() {
        let order = RuleSource::priority_order();
        assert_eq!(order[0], RuleSource::Session);
        assert_eq!(order[order.len() - 1], RuleSource::UserSettings);
    }

    #[test]
    fn test_permission_rule_matches() {
        // 精确匹配
        let rule = PermissionRule::allow(RuleSource::UserSettings, "Read");
        assert!(rule.matches("Read", None));
        assert!(!rule.matches("Write", None));

        // 精确命令匹配
        let rule = PermissionRule::allow(RuleSource::UserSettings, "Bash(npm test)");
        assert!(rule.matches("Bash", Some("npm test")));
        assert!(!rule.matches("Bash", Some("npm run build")));

        // 前缀匹配
        let rule = PermissionRule::allow(RuleSource::UserSettings, "Bash(npm:*)");
        assert!(rule.matches("Bash", Some("npm test")));
        assert!(rule.matches("Bash", Some("npm run build")));
        assert!(!rule.matches("Bash", Some("yarn test")));

        // 通配符匹配
        let rule = PermissionRule::allow(RuleSource::UserSettings, "Bash(git *)");
        assert!(rule.matches("Bash", Some("git status")));
        assert!(rule.matches("Bash", Some("git commit -m test")));
        assert!(!rule.matches("Bash", Some("git")));
    }

    #[test]
    fn test_permission_context() {
        let mut ctx = ToolPermissionContext::with_defaults();

        // 添加规则
        ctx.add_allow_rule(RuleSource::UserSettings, "Read");
        ctx.add_deny_rule(RuleSource::ProjectSettings, "Bash(rm -rf *)");

        // 获取优先级排序的规则
        let deny_rules = ctx.get_rules_by_priority(PermissionAction::Deny);
        assert_eq!(deny_rules.len(), 1);
        assert_eq!(deny_rules[0].source, RuleSource::ProjectSettings);

        // 测试安全工具
        assert!(ctx.is_safe_tool("Read"));
        assert!(!ctx.is_safe_tool("Write"));
    }

    #[test]
    fn test_permission_update() {
        let ctx = ToolPermissionContext::with_defaults();

        // 创建更新
        let update = PermissionUpdate::add_rules(
            RuleSource::UserSettings,
            vec![PermissionRule::allow(RuleSource::UserSettings, "Glob")],
        );

        // 应用更新
        let result = apply_permission_update(&ctx, &update);

        // 验证结果
        assert!(result.has_persistable_update);
        assert!(result
            .new_context
            .allow_rules
            .get(&RuleSource::UserSettings)
            .is_some());

        // 原始上下文应该保持不变（不可变性）
        assert!(ctx.allow_rules.get(&RuleSource::UserSettings).is_none());
    }

    #[test]
    fn test_wildcard_match() {
        // 简单通配符匹配
        assert!(PermissionRule::simple_wildcard_match("git *", "git status"));
        assert!(PermissionRule::simple_wildcard_match(
            "git *",
            "git commit -m test"
        ));
        assert!(!PermissionRule::simple_wildcard_match(
            "git *",
            "svn status"
        ));

        // 多段通配符
        assert!(PermissionRule::simple_wildcard_match(
            "* /etc/*",
            "cat /etc/passwd"
        ));
        assert!(PermissionRule::simple_wildcard_match(
            "* /etc/*",
            "echo hello > /etc/hosts"
        ));

        // 精确匹配
        assert!(PermissionRule::simple_wildcard_match(
            "npm test", "npm test"
        ));
        assert!(!PermissionRule::simple_wildcard_match(
            "npm test",
            "npm run test"
        ));
    }
}
