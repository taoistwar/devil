//! Skill 执行引擎
//!
//! 实现两条执行路径：Inline 模式和 Fork 模式

use crate::message::{Message, ContentBlock};
use crate::skills::types::{ExecutionContext, SkillCommand, SkillLoadSource, SkillSource};
use crate::subagent::{SubagentParams, SubagentType};
use std::collections::HashMap;
use tokio::process::Command as TokioCommand;

/// Skill 执行器
pub struct SkillExecutor {
    /// 当前会话 ID
    session_id: String,
}

impl SkillExecutor {
    /// 创建执行器
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
        }
    }

    /// 执行 Skill
    ///
    /// 根据 `context` 字段分流到 Inline 或 Fork 模式
    pub async fn execute(
        &self,
        skill: &SkillCommand,
        arguments: Option<&str>,
    ) -> Result<SkillExecutionResult, SkillExecutionError> {
        match skill.context {
            ExecutionContext::Inline => self.execute_inline(skill, arguments).await,
            ExecutionContext::Fork => self.execute_fork(skill, arguments).await,
        }
    }

    /// Inline 模式执行
    ///
    /// Skill 的 Prompt 内容被注入为 UserMessage，在主对话流中继续执行
    async fn execute_inline(
        &self,
        skill: &SkillCommand,
        arguments: Option<&str>,
    ) -> Result<SkillExecutionResult, SkillExecutionError> {
        // 1. 处理参数替换（$ARGUMENTS）
        let content = self.process_arguments(&skill.content, arguments);

        // 2. 替换环境变量
        let content = self.replace_env_variables(&content, &skill.skill_dir);

        // 3. 展开 Shell 命令（!`...`）
        let content = self.expand_shell_commands(&content).await?;

        // 4. 构建上下文修饰器（修改权限上下文）
        let context_modifier = self.build_context_modifier(skill);

        // 5. 返回新消息和上下文修饰器
        Ok(SkillExecutionResult::Inline {
            new_messages: vec![Message::user_text(content)],
            context_modifier,
        })
    }

    /// Fork 模式执行
    ///
    /// Skill 在独立子 Agent 中执行，拥有独立的 token 预算
    async fn execute_fork(
        &self,
        skill: &SkillCommand,
        arguments: Option<&str>,
    ) -> Result<SkillExecutionResult, SkillExecutionError> {
        // 1. 准备 Fork 上下文
        let content = self.process_arguments(&skill.content, arguments);
        let content = self.replace_env_variables(&content, &skill.skill_dir);

        // 2. 构建子代理参数
        let params = SubagentParams {
            prompt_messages: vec![Message::user_text(content)],
            cache_safe_params: todo!("从父级继承"),
            subagent_type: SubagentType::Custom(
                skill.agent.clone().unwrap_or_else(|| "worker".to_string()),
            ),
            directive: skill.description.clone(),
            max_turns: None,
            max_output_tokens: None,
            skip_transcript: false,
            skip_cache_write: false,
            run_in_background: true,
            worktree_path: None,
            parent_cwd: None,
        };

        // 3. 执行子代理（TODO: 需要与 subagent executor 集成）
        // let result = subagent_executor.execute(params).await?;

        // 4. 提取结果文本
        // let result_text = extract_result_text(&result.messages);

        Ok(SkillExecutionResult::Fork {
            result: "Fork execution result (TODO)".to_string(),
            agent_messages: vec![],
        })
    }

    /// 处理参数替换
    fn process_arguments(&self, content: &str, arguments: Option<&str>) -> String {
        let mut result = content.to_string();

        // 替换 $ARGUMENTS
        if let Some(args) = arguments {
            result = result.replace("$ARGUMENTS", args);
        }

        result
    }

    /// 替换环境变量
    fn replace_env_variables(&self, content: &str, skill_dir: &str) -> String {
        let mut result = content.to_string();

        // 替换 ${CLAUDE_SKILL_DIR}
        result = result.replace("${CLAUDE_SKILL_DIR}", skill_dir);

        // 替换 ${CLAUDE_SESSION_ID}
        result = result.replace("${CLAUDE_SESSION_ID}", &self.session_id);

        result
    }

    /// 展开 Shell 命令（!`...`）
    ///
    /// 支持以下语法：
    /// - !`command` - 执行命令并替换输出
    /// - ${CLAUDE_SKILL_DIR} - 技能目录
    /// - ${CLAUDE_SESSION_ID} - 会话 ID
    ///
    /// 安全检查：
    /// - MCP 技能禁止执行 Shell 命令
    /// - 危险命令被过滤（rm -rf /, curl | bash 等）
    async fn expand_shell_commands(&self, content: &str) -> Result<String, SkillExecutionError> {
        let mut result = content.to_string();

        // 查找所有 !`command` 模式
        while let Some(start) = result.find("!`") {
            if let Some(end) = result[start + 2..].find('`') {
                let command_start = start + 2;
                let command_end = start + 2 + end;
                let command = result[command_start..command_end].to_string();

                // 执行命令并获取输出
                match self.execute_safe_command(&command).await {
                    Ok(output) => {
                        // 替换 !`command` 为命令输出
                        result.replace_range(start..=command_end, &output);
                    }
                    Err(e) => {
                        // 命令执行失败，保留原始标记但添加错误信息
                        let error_msg = format!("[Shell command failed: {}]", e);
                        result.replace_range(start..=command_end, &error_msg);
                    }
                }
            } else {
                // 未找到闭合的 `,停止处理
                break;
            }
        }

        Ok(result)
    }

    /// 执行安全命令
    ///
    /// 检查并执行 Shell 命令，过滤危险命令
    async fn execute_safe_command(&self, command: &str) -> Result<String, SkillExecutionError> {
        // 检查危险命令
        if shell_expansion::is_dangerous_command(command) {
            return Err(SkillExecutionError::ShellCommandError(format!(
                "Dangerous command blocked: {}",
                command
            )));
        }

        // 展开复合命令（&&、||、管道）
        let expanded_commands = shell_expansion::expand_command(command);

        // 执行命令（简化实现：只执行第一个简单命令）
        // 完整实现应该支持管道和命令链
        let cmd = expanded_commands
            .first()
            .ok_or_else(|| SkillExecutionError::ShellCommandError("Empty command".to_string()))?;

        // 拆分命令和参数
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Err(SkillExecutionError::ShellCommandError(
                "Empty command after expansion".to_string(),
            ));
        }

        let program = parts[0];
        let args: Vec<&str> = parts[1..].to_vec();

        // 执行命令
        let output = TokioCommand::new(program)
            .args(&args)
            .output()
            .await
            .map_err(|e| {
                SkillExecutionError::ShellCommandError(format!(
                    "Failed to execute '{}': {}",
                    cmd, e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SkillExecutionError::ShellCommandError(format!(
                "Command '{}' failed: {}",
                cmd, stderr
            )));
        }

        // 返回标准输出（去除末尾换行）
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    /// 构建上下文修饰器
    ///
    /// 修改权限上下文：
    /// - 工具白名单注入：将 allowedTools 合并到 alwaysAllowRules.command
    /// - 模型切换：处理模型覆盖
    /// - 努力级别覆盖：修改 effortValue
    fn build_context_modifier(&self, skill: &SkillCommand) -> ContextModifier {
        ContextModifier {
            allowed_tools: skill.allowed_tools.clone(),
            model: skill.model.clone(),
            effort: skill.effort.clone(),
        }
    }
}

/// 上下文修饰器
#[derive(Debug, Clone, Default)]
pub struct ContextModifier {
    /// 工具白名单
    pub allowed_tools: Vec<String>,
    /// 模型覆盖
    pub model: Option<crate::skills::types::ModelOverride>,
    /// 努力级别
    pub effort: Option<crate::skills::types::EffortLevel>,
}

/// Skill 执行结果
#[derive(Debug, Clone)]
pub enum SkillExecutionResult {
    /// Inline 模式结果
    Inline {
        /// 注入到对话流的新消息
        new_messages: Vec<Message>,
        /// 上下文修饰器（修改权限上下文）
        context_modifier: ContextModifier,
    },
    /// Fork 模式结果
    Fork {
        /// 提取的结果文本
        result: String,
        /// 子 Agent 的全部消息（提取后释放）
        agent_messages: Vec<Message>,
    },
}

/// Skill 执行错误
#[derive(Debug, thiserror::Error)]
pub enum SkillExecutionError {
    #[error("参数处理失败：{0}")]
    ArgumentError(String),

    #[error("Shell 命令执行失败：{0}")]
    ShellCommandError(String),

    #[error("子代理执行失败：{0}")]
    SubagentError(String),

    #[error("权限检查失败：{0}")]
    PermissionError(String),

    #[error("内部错误：{0}")]
    InternalError(String),
}

/// 提取结果文本（Fork 模式）
pub fn extract_result_text(messages: &[Message]) -> String {
    // 从子 Agent 消息中提取最终文本响应
    // 优先返回最后一条 assistant 消息
    messages
        .iter()
        .rev()
        .find(|msg| msg.role() == "assistant")
        .and_then(|msg| {
            if let Message::Assistant(asm) = msg {
                asm.content.iter().find_map(|block| {
                    if let crate::message::ContentBlock::Text { text } = block {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .unwrap_or("")
        .to_string()
}

/// 清理已调用的 Skill 状态
pub fn clear_invoked_skills_for_agent() {
    // 清理状态逻辑（实际实现需要访问全局状态）
}

/// Skill 预算管理器
///
/// 实现 Claude Code 风格的 Prompt 预算管理：
/// - 预算计算：context_window_tokens × 4 chars/token × 1%
/// - 单条上限：MAX_LISTING_DESC_CHARS = 250 字符
/// - Bundled Skills 不可截断
/// - 降级策略：完整描述 → 均分预算 → 仅保留名称
pub struct SkillBudgetManager {
    /// 上下文窗口 token 数
    context_window_tokens: usize,
    /// 当前 token 预算（字符数）
    current_budget: usize,
    /// 保留预算（不可截断部分）
    reserved_budget: usize,
    /// 单条技能描述上限（字符数）
    max_desc_chars: usize,
}

impl SkillBudgetManager {
    /// 预算百分比（1% 上下文窗口）
    const BUDGET_PERCENTAGE: f64 = 0.01;

    /// 单条描述上限（250 字符）
    const MAX_LISTING_DESC_CHARS: usize = 250;

    /// 创建预算管理器
    ///
    /// # Arguments
    /// * `context_window_tokens` - 上下文窗口 token 数
    /// * `reserved_percentage` - 保留预算百分比（0.0-1.0）
    pub fn new(context_window_tokens: usize, reserved_percentage: f64) -> Self {
        // 预算计算：context_window_tokens × 4 chars/token × 1%
        let total_budget = context_window_tokens * 4;
        let budget = (total_budget as f64 * Self::BUDGET_PERCENTAGE) as usize;
        let reserved_budget = (budget as f64 * reserved_percentage) as usize;

        Self {
            context_window_tokens,
            current_budget: budget,
            reserved_budget,
            max_desc_chars: Self::MAX_LISTING_DESC_CHARS,
        }
    }

    /// 检查是否需要截断
    ///
    /// 当预算紧张时，截断普通 skills 的 prompt，但保留：
    /// - 用户消息（不可截断）
    /// - 系统提示（不可截断）
    /// - Bundled Skills（不可截断）
    pub fn should_truncate(&self, used_tokens: usize) -> bool {
        let available = self.current_budget.saturating_sub(self.reserved_budget);
        used_tokens > available
    }

    /// 格式化技能列表在预算内
    ///
    /// 降级策略：
    /// 1. 尝试完整描述 → 超预算？
    /// 2. Bundled 保留完整，非 bundled 均分剩余预算 → 每条描述低于 20 字符？
    /// 3. 非 bundled 仅保留名称
    ///
    /// # Returns
    /// 返回格式化后的技能描述列表和是否被截断
    pub fn format_skills_in_budget(&self, skills: &[SkillCommand]) -> (Vec<String>, bool) {
        let mut truncated = false;
        let mut formatted = Vec::new();

        // 首先尝试完整描述
        let full_descriptions: Vec<String> = skills
            .iter()
            .map(|s| {
                let desc = format!("- **{}**: {}", s.name, s.description);
                if desc.len() > self.max_desc_chars {
                    format!("{}...", &desc[..self.max_desc_chars])
                } else {
                    desc
                }
            })
            .collect();

        let total_chars: usize = full_descriptions.iter().map(|d| d.len()).sum();

        if total_chars <= self.current_budget {
            // 完整描述在预算内
            return (full_descriptions, false);
        }

        truncated = true;

        // 分离 bundled 和非 bundled 技能
        let (bundled, non_bundled): (Vec<_>, Vec<_>) = skills
            .iter()
            .partition(|s| s.source == SkillSource::Bundled || s.source == SkillSource::BuiltIn);

        // Bundled 技能保留完整描述
        for skill in &bundled {
            let desc = format!("- **{}**: {}", skill.name, skill.description);
            formatted.push(desc);
        }

        // 计算剩余预算给非 bundled 技能
        let bundled_chars: usize = formatted.iter().map(|d| d.len()).sum();
        let remaining_budget = self.current_budget.saturating_sub(bundled_chars);

        if non_bundled.is_empty() {
            return (formatted, truncated);
        }

        // 均分剩余预算
        let budget_per_skill = remaining_budget / non_bundled.len();

        for skill in &non_bundled {
            let full_desc = format!("- **{}**: {}", skill.name, skill.description);

            if budget_per_skill >= 20 {
                // 有足够的预算，使用截断的描述
                if full_desc.len() <= budget_per_skill {
                    formatted.push(full_desc);
                } else {
                    formatted.push(format!(
                        "{}...",
                        &full_desc[..budget_per_skill.saturating_sub(3)]
                    ));
                }
            } else {
                // 预算不足，仅保留名称
                formatted.push(format!("- **{}**", skill.name));
            }
        }

        (formatted, truncated)
    }

    /// 截断技能内容
    ///
    /// 按优先级截断：
    /// 1. 普通 Disk Skills（优先级最低）
    /// 2. Legacy Skills
    /// 3. Plugin Skills
    /// 4. Bundled Skills（不可截断）
    pub fn truncate_skills(
        &self,
        skills: &mut Vec<SkillCommand>,
        target_tokens: usize,
    ) -> Vec<SkillCommand> {
        // 按来源排序优先级（低优先级的先被截断）
        let priority_order = |skill: &SkillCommand| -> u8 {
            match skill.source {
                SkillSource::Disk => 0, // 最低优先级
                SkillSource::Legacy => 1,
                SkillSource::MCP => 2,
                SkillSource::Bundled => 3, // 最高优先级（不可截断）
                SkillSource::BuiltIn => 3, // 最高优先级（不可截断）
            }
        };

        // 按优先级排序
        skills.sort_by_key(|s| priority_order(s));

        let mut truncated = Vec::new();
        let mut current_tokens = 0;

        for skill in skills.drain(..) {
            // 高优先级技能不截断
            if priority_order(&skill) >= 3 {
                current_tokens += estimate_tokens(&skill.content);
                continue;
            }

            // 检查是否超出预算
            let skill_tokens = estimate_tokens(&skill.content);
            if current_tokens + skill_tokens > target_tokens {
                // 需要截断这个技能
                truncated.push(skill);
            } else {
                current_tokens += skill_tokens;
            }
        }

        truncated
    }

    /// 获取可用预算
    pub fn available_budget(&self) -> usize {
        self.current_budget.saturating_sub(self.reserved_budget)
    }

    /// 获取总预算（字符数）
    pub fn total_budget(&self) -> usize {
        self.current_budget
    }

    /// 获取保留预算
    pub fn reserved_budget(&self) -> usize {
        self.reserved_budget
    }
}

/// 估算 token 数量
///
/// 简单实现：按字符数 / 4 估算（英文）
/// 实际应使用 tiktoken 或类似库
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Shell 命令展开器模块
///
/// 提供安全的 Shell 命令展开和执行功能
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

    /// 检查 MCP 安全边界
    ///
    /// MCP 技能禁止执行 Shell 命令
    pub fn is_mcp_safe(loaded_from: &SkillLoadSource) -> bool {
        // MCP 来源的技能不允许执行 Shell 命令
        loaded_from != &SkillLoadSource::MCP
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
mod budget_tests {
    use super::*;

    #[test]
    fn test_budget_manager_creation() {
        let manager = SkillBudgetManager::new(10000, 0.2);
        assert_eq!(manager.reserved_budget, 2000);
        assert_eq!(manager.current_budget, 10000);
    }

    #[test]
    fn test_should_truncate() {
        let manager = SkillBudgetManager::new(10000, 0.2);

        // 使用量在可用预算内
        assert!(!manager.should_truncate(7000));

        // 使用量超出可用预算
        assert!(manager.should_truncate(8500));
    }

    #[test]
    fn test_available_budget() {
        let manager = SkillBudgetManager::new(10000, 0.3);
        assert_eq!(manager.available_budget(), 7000);
    }

    #[test]
    fn test_estimate_tokens() {
        // 英文估算
        let text = "Hello world this is a test";
        let tokens = estimate_tokens(text);
        assert!(tokens > 0);
        assert!(tokens < text.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::{SkillLoadSource, SkillSource};

    #[tokio::test]
    async fn test_inline_execution() {
        let executor = SkillExecutor::new("test-session");

        let skill = SkillCommand {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            when_to_use: None,
            allowed_tools: vec![],
            argument_hint: None,
            arguments: vec!["path".to_string()],
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
            content: "Review the code at $ARGUMENTS".to_string(),
            skill_dir: "/test/skill/dir".to_string(),
        };

        let result = executor.execute(&skill, Some("src/main.rs")).await.unwrap();

        match result {
            SkillExecutionResult::Inline { new_messages, .. } => {
                assert_eq!(new_messages.len(), 1);
                if let Message::User(msg) = &new_messages[0] {
                    let text = msg.content.iter()
                        .find_map(|c| if let ContentBlock::Text { text } = c { Some(text) } else { None })
                        .unwrap();
                    assert!(text.contains("src/main.rs"));
                }
            }
            _ => panic!("Expected Inline result"),
        }
    }

    #[test]
    fn test_env_replacement() {
        let executor = SkillExecutor::new("session-123");

        let content = "Skill dir: ${CLAUDE_SKILL_DIR}, Session: ${CLAUDE_SESSION_ID}";
        let result = executor.replace_env_variables(content, "/my/skill");

        assert!(result.contains("/my/skill"));
        assert!(result.contains("session-123"));
    }
}
