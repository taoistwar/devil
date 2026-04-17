//! Skill 执行引擎
//! 
//! 实现两条执行路径：Inline 模式和 Fork 模式

use crate::skills::types::{SkillCommand, ExecutionContext, SkillSource};
use crate::message::Message;
use crate::subagent::{SubagentParams, SubagentType};
use std::collections::HashMap;

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
            ExecutionContext::Inline => {
                self.execute_inline(skill, arguments).await
            }
            ExecutionContext::Fork => {
                self.execute_fork(skill, arguments).await
            }
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
            new_messages: vec![Message::user(content)],
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
            prompt_messages: vec![Message::user(content)],
            cache_safe_params: todo!("从父级继承"),
            subagent_type: SubagentType::Custom(
                skill.agent.clone().unwrap_or_else(|| "worker".to_string())
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
    async fn expand_shell_commands(&self, content: &str) -> Result<String, SkillExecutionError> {
        // TODO: 实现 Shell 命令展开
        // 匹配 !`command` 并执行
        Ok(content.to_string())
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
        .find(|msg| msg.role == "assistant")
        .and_then(|msg| msg.content.as_text())
        .unwrap_or("")
        .to_string()
}

/// 清理已调用的 Skill 状态
pub fn clear_invoked_skills_for_agent() {
    // 清理状态逻辑（实际实现需要访问全局状态）
}

/// Skill 预算管理器
pub struct SkillBudgetManager {
    /// 当前 token 预算
    current_budget: usize,
    /// 保留预算（不可截断部分）
    reserved_budget: usize,
}

impl SkillBudgetManager {
    /// 创建预算管理器
    pub fn new(total_budget: usize, reserved_percentage: f64) -> Self {
        let reserved_budget = (total_budget as f64 * reserved_percentage) as usize;
        Self {
            current_budget: total_budget,
            reserved_budget,
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
                SkillSource::Disk => 0,       // 最低优先级
                SkillSource::Legacy => 1,
                SkillSource::MCP => 2,
                SkillSource::Bundled => 3,    // 最高优先级（不可截断）
                SkillSource::BuiltIn => 3,    // 最高优先级（不可截断）
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
}

/// 估算 token 数量
/// 
/// 简单实现：按字符数 / 4 估算（英文）
/// 实际应使用 tiktoken 或类似库
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// 截断长内容
pub fn truncate_content(content: &str, max_tokens: usize) -> String {
    let estimated = estimate_tokens(content);
    if estimated <= max_tokens {
        return content.to_string();
    }
    
    // 按比例截断
    let truncate_ratio = max_tokens as f64 / estimated as f64;
    let truncate_chars = (content.len() as f64 * truncate_ratio) as usize;
    
    format!("{}...", &content[..truncate_chars.max(0)])
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
    
    #[test]
    fn test_truncate_content() {
        let content = "This is a long content that needs to be truncated";
        let truncated = truncate_content(content, 5);
        assert!(truncated.len() < content.len());
        assert!(truncated.ends_with("..."));
    }
    
    #[test]
    fn test_truncate_content_no_truncation() {
        let content = "Short content";
        let truncated = truncate_content(content, 1000);
        assert_eq!(truncated, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::{SkillSource, SkillLoadSource};
    
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
                assert!(new_messages[0].content.as_text().unwrap().contains("src/main.rs"));
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
