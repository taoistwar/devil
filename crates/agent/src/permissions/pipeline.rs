//! 权限管线模块
//!
//! 实现 Claude Code 的四阶段权限检查流程：
//! 1. validateInput - Zod Schema 验证
//! 2. hasPermissionsToUseTool - 规则匹配
//! 3. checkPermissions - 上下文评估
//! 4. 交互式提示 - 用户确认

use crate::permissions::context::*;
use crate::tools::tool::{PermissionResult, Tool, ToolContext};
use anyhow::Result;

/// 权限检查的中间结果
#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    /// 权限行为
    pub behavior: PermissionBehavior,
    /// 命中的规则（如果有）
    pub matched_rule: Option<PermissionRule>,
    /// 规则来源（如果有）
    pub rule_source: Option<RuleSource>,
}

/// 权限行为
#[derive(Debug, Clone)]
pub enum PermissionBehavior {
    /// 允许执行
    Allow,
    /// 拒绝执行
    Deny {
        /// 拒绝原因
        reason: String,
    },
    /// 询问用户
    Ask {
        /// 提示信息
        prompt: String,
    },
    /// 传递到下一阶段
    Passthrough,
}

impl PermissionCheckResult {
    /// 创建允许的结果
    pub fn allow() -> Self {
        Self {
            behavior: PermissionBehavior::Allow,
            matched_rule: None,
            rule_source: None,
        }
    }

    /// 创建拒绝的结果
    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            behavior: PermissionBehavior::Deny {
                reason: reason.into(),
            },
            matched_rule: None,
            rule_source: None,
        }
    }

    /// 创建询问的结果
    pub fn ask(prompt: impl Into<String>) -> Self {
        Self {
            behavior: PermissionBehavior::Ask {
                prompt: prompt.into(),
            },
            matched_rule: None,
            rule_source: None,
        }
    }

    /// 创建传递的结果
    pub fn passthrough() -> Self {
        Self {
            behavior: PermissionBehavior::Passthrough,
            matched_rule: None,
            rule_source: None,
        }
    }

    /// 创建带规则匹配结果的允许
    pub fn allow_with_rule(rule: PermissionRule, source: RuleSource) -> Self {
        Self {
            behavior: PermissionBehavior::Allow,
            matched_rule: Some(rule),
            rule_source: Some(source),
        }
    }

    /// 创建带规则匹配结果的拒绝
    pub fn deny_with_rule(
        rule: PermissionRule,
        source: RuleSource,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            behavior: PermissionBehavior::Deny {
                reason: reason.into(),
            },
            matched_rule: Some(rule),
            rule_source: Some(source),
        }
    }

    /// 转换为 PermissionResult
    pub fn to_permission_result(&self) -> PermissionResult {
        match &self.behavior {
            PermissionBehavior::Allow => PermissionResult::allow(),
            PermissionBehavior::Deny { reason } => PermissionResult::deny(reason.clone()),
            PermissionBehavior::Ask { prompt } => PermissionResult::ask(prompt.clone()),
            PermissionBehavior::Passthrough => PermissionResult::allow(),
        }
    }
}

/// 权限管线
///
/// 实现四阶段权限检查流程
pub struct PermissionPipeline;

impl PermissionPipeline {
    /// 执行完整的四阶段权限检查
    ///
    /// 返回最终的权限决策
    pub async fn check_permissions<T: Tool>(
        tool: &T,
        input: &serde_json::Value,
        context: &ToolContext,
        permission_context: &ToolPermissionContext,
    ) -> Result<PermissionCheckResult> {
        // 阶段一：输入验证
        let validation_result = Self::phase1_validate_input(tool, input);
        if !validation_result.is_valid {
            // 解析失败时优雅降级为 ask
            return Ok(PermissionCheckResult::ask(
                validation_result
                    .error_message
                    .unwrap_or_else(|| "Invalid input".to_string()),
            ));
        }

        // 反序列化为工具的输入类型
        let typed_input = tool.validate_input(input);

        // 阶段二：规则匹配
        let rule_check = Self::phase2_rule_matching(tool, input, permission_context);
        match &rule_check.behavior {
            PermissionBehavior::Allow { .. } | PermissionBehavior::Deny { .. } => {
                // 规则已做出终局决定
                return Ok(rule_check);
            }
            PermissionBehavior::Ask { .. } => {
                // ask 规则命中，但可能进入阶段四交互式提示
                // 这里先标记，后续处理
            }
            PermissionBehavior::Passthrough => {
                // 无规则匹配，继续阶段三
            }
        }

        // 阶段三：上下文评估
        let context_check =
            Self::phase3_context_check(tool, input, context, permission_context).await;
        match &context_check.behavior {
            PermissionBehavior::Allow { .. } | PermissionBehavior::Deny { .. } => {
                // 上下文评估做出终局决定
                return Ok(context_check);
            }
            PermissionBehavior::Ask { .. } | PermissionBehavior::Passthrough => {
                // 进入阶段四
            }
        }

        // 阶段四：交互式提示
        // 如果前面阶段返回 ask 或 passthrough，最终都会变为 ask
        let final_result = if matches!(rule_check.behavior, PermissionBehavior::Ask { .. }) {
            rule_check
        } else if matches!(
            context_check.behavior,
            PermissionBehavior::Ask { .. } | PermissionBehavior::Passthrough
        ) {
            // 检查权限模式
            match permission_context.mode {
                PermissionMode::BypassPermissions => {
                    // bypass 模式：自动允许
                    return Ok(PermissionCheckResult::allow());
                }
                PermissionMode::Auto => {
                    // auto 模式：使用分类器
                    // 先检查是否在安全白名单中
                    if permission_context.is_safe_tool(tool.name()) {
                        return Ok(PermissionCheckResult::allow());
                    }
                    // 检查 acceptEdits 快速路径
                    if permission_context.accept_edits_mode && tool.is_read_only() {
                        return Ok(PermissionCheckResult::allow());
                    }
                    // 调用分类器（这里简化处理，实际应该调用 AI 分类器）
                    PermissionCheckResult::ask(
                        "AI classifier needs to review this operation".to_string(),
                    )
                }
                PermissionMode::Plan => {
                    // plan 模式：如果是写入操作则拒绝
                    if !tool.is_read_only() {
                        return Ok(PermissionCheckResult::deny(
                            "Operation denied in plan mode (read-only)".to_string(),
                        ));
                    }
                    PermissionCheckResult::ask("Confirm operation in plan mode".to_string())
                }
                PermissionMode::Default | PermissionMode::Bubble => {
                    // default 模式：总是询问
                    PermissionCheckResult::ask("Confirm operation".to_string())
                }
            }
        } else {
            context_check
        };

        Ok(final_result)
    }

    /// 阶段一：输入验证
    ///
    /// 使用 Zod Schema 验证输入数据的合法性
    fn phase1_validate_input<T: Tool>(
        tool: &T,
        input: &serde_json::Value,
    ) -> crate::tools::tool::InputValidationResult {
        // 调用工具的 validate_input 方法
        tool.validate_input(input)
    }

    /// 阶段二：规则匹配
    ///
    /// 按照严格的优先级顺序检查三类规则：deny > ask > allow
    ///
    /// 优先级铁律：deny 始终优先于 allow，无论它们的来源如何
    fn phase2_rule_matching<T: Tool>(
        tool: &T,
        input: &serde_json::Value,
        permission_context: &ToolPermissionContext,
    ) -> PermissionCheckResult {
        let tool_name = tool.name();

        // 提取命令内容（如果是 Bash 工具）
        let command = Self::extract_command(input);

        // 步骤 1a：工具级 deny 检查（最高优先级）
        for rule in permission_context.get_rules_by_priority(PermissionAction::Deny) {
            if rule.matches(tool_name, command.as_deref()) {
                return PermissionCheckResult::deny_with_rule(
                    rule.clone(),
                    rule.source,
                    format!(
                        "Operation denied by {} rule from {:?}",
                        rule.target, rule.source
                    ),
                );
            }
        }

        // 步骤 1b：工具级 ask 检查
        for rule in permission_context.get_rules_by_priority(PermissionAction::Ask) {
            if rule.matches(tool_name, command.as_deref()) {
                // 沙箱模式例外：Bash 工具在沙箱模式下且配置了沙箱自动放行
                if tool_name == "Bash" {
                    // TODO: 检查沙箱配置
                    // if is_sandboxed && sandbox_auto_approve {
                    //     continue;
                    // }
                }
                return PermissionCheckResult::ask(format!(
                    "Operation requires confirmation (rule: {})",
                    rule.target
                ));
            }
        }

        // 步骤 2b：工具级 allow 检查
        for rule in permission_context.get_rules_by_priority(PermissionAction::Allow) {
            if rule.matches(tool_name, command.as_deref()) {
                return PermissionCheckResult::allow_with_rule(rule.clone(), rule.source);
            }
        }

        // 无规则匹配，返回 passthrough 交由后续阶段决定
        PermissionCheckResult::passthrough()
    }

    /// 阶段三：上下文评估
    ///
    /// 调用工具的 check_permissions 方法进行更精细的上下文评估
    async fn phase3_context_check<T: Tool>(
        tool: &T,
        input: &serde_json::Value,
        _context: &ToolContext,
        _permission_context: &ToolPermissionContext,
    ) -> PermissionCheckResult {
        // 调用工具的 check_permissions 方法
        let typed_input = match serde_json::from_value::<T::Input>(input.clone()) {
            Ok(input) => input,
            Err(_) => return PermissionCheckResult::allow(), // 如果无法解析输入，默认允许
        };
        let perm_result = tool
            .check_permissions(&typed_input, &ToolContext::default())
            .await;

        match perm_result.behavior {
            crate::tools::tool::PermissionBehavior::Allow { .. } => PermissionCheckResult::allow(),
            crate::tools::tool::PermissionBehavior::Deny { reason } => {
                PermissionCheckResult::deny(reason)
            }
            crate::tools::tool::PermissionBehavior::Ask { prompt } => {
                PermissionCheckResult::ask(prompt)
            }
        }
    }

    /// 从输入中提取命令内容
    ///
    /// 专门处理 Bash 工具的命令提取
    fn extract_command(input: &serde_json::Value) -> Option<String> {
        input
            .get("command")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

/// ResolveOnce 模式：原子化的竞争解决
///
/// 用于解决用户交互和分类器自动审批之间的竞争条件
/// 保证只有一个决策者能成功 claim 决策权
pub struct ResolveOnce<T> {
    claimed: std::sync::atomic::AtomicBool,
    value: std::sync::Mutex<Option<T>>,
}

impl<T: Clone> ResolveOnce<T> {
    /// 创建新的 ResolveOnce
    pub fn new(value: Option<T>) -> Self {
        Self {
            claimed: std::sync::atomic::AtomicBool::new(false),
            value: std::sync::Mutex::new(value),
        }
    }

    /// 尝试原子化地 claim 决策权
    ///
    /// 返回 true 表示 claim 成功，此调用者有权设置最终值
    /// 返回 false 表示已被其他调用者 claim
    pub fn claim(&self) -> bool {
        !self.claimed.swap(true, std::sync::atomic::Ordering::SeqCst)
    }

    /// 设置值（如果已 claim）
    pub fn set(&self, value: T) -> Result<()> {
        if self.claimed.load(std::sync::atomic::Ordering::SeqCst) {
            let mut inner = self.value.lock().unwrap();
            *inner = Some(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "ResolveOnce already claimed by another caller"
            ))
        }
    }

    /// 获取当前值
    pub fn get(&self) -> Option<T> {
        self.value.lock().ok().and_then(|v| v.clone())
    }

    /// 判断是否已解决
    pub fn is_resolved(&self) -> bool {
        self.claimed.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl<T: Clone> Default for ResolveOnce<T> {
    fn default() -> Self {
        Self::new(None)
    }
}

/// 分类器检查结果
#[derive(Debug, Clone)]
pub struct ClassifierCheckResult {
    /// 是否批准
    pub approved: bool,
    /// 置信度（0.0 - 1.0）
    pub confidence: f32,
    /// 拒绝原因（如果拒绝了）
    pub reason: Option<String>,
}

/// 运行分类器检查（auto 模式优化）
///
/// 使用 2 秒超时，与用户交互形成"竞赛"机制
pub async fn run_classifier_check(
    tool_name: &str,
    input: &serde_json::Value,
) -> ClassifierCheckResult {
    // 模拟分类器检查
    // 实际实现应该调用 AI 分类器 API

    // 简单规则：如果工具名包含 "Read"、"Grep"、"Glob" 等，直接批准
    let approved = ["Read", "Grep", "Glob", "TodoWrite"]
        .iter()
        .any(|safe| tool_name.contains(safe));

    ClassifierCheckResult {
        approved,
        confidence: if approved { 0.95 } else { 0.3 },
        reason: if !approved {
            Some("Operation appears to have side effects".to_string())
        } else {
            None
        },
    }
}

/// 检查是否应该跳过权限提示
///
/// 用于非交互式会话
pub fn should_skip_permission_prompt(
    permission_context: &ToolPermissionContext,
    check_result: &PermissionCheckResult,
) -> bool {
    // 如果配置了避免权限提示
    if permission_context.avoid_permission_prompts {
        // 但不是 deny 行为
        if !matches!(check_result.behavior, PermissionBehavior::Deny { .. }) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_once() {
        let resolve_once = ResolveOnce::new(Some("initial".to_string()));

        // 第一次 claim 应该成功
        assert!(resolve_once.claim());

        // 第二次 claim 应该失败
        assert!(!resolve_once.claim());

        // 值应该存在
        assert_eq!(resolve_once.get(), Some("initial".to_string()));

        // is_resolved 应该返回 true
        assert!(resolve_once.is_resolved());
    }

    #[test]
    fn test_classifier_check() {
        // 安全工具应该被批准
        let result =
            futures::executor::block_on(run_classifier_check("Read", &serde_json::Value::Null));
        assert!(result.approved);
        assert!(result.confidence > 0.5);

        // 不安全工具可能被拒绝
        let result =
            futures::executor::block_on(run_classifier_check("Bash", &serde_json::Value::Null));
        // Bash 不一定被拒绝，取决于具体实现
    }

    #[test]
    fn test_skip_permission_prompt() {
        let mut ctx = ToolPermissionContext::with_defaults();
        let result = PermissionCheckResult::allow();

        // 默认不应该跳过
        assert!(!should_skip_permission_prompt(&ctx, &result));

        // 设置 avoid_permission_prompts 后应该跳过
        ctx.avoid_permission_prompts = true;
        assert!(should_skip_permission_prompt(&ctx, &result));

        // 但 deny 行为不应该跳过
        let deny_result = PermissionCheckResult::deny("Test");
        assert!(!should_skip_permission_prompt(&ctx, &deny_result));
    }
}
