//! 子代理执行引擎
//! 
//! 实现子代理的启动、执行和管理

use crate::subagent::types::{SubagentParams, SubagentResult, SubagentType, SubagentDefinition, Usage};
use crate::subagent::context_inheritance;
use crate::subagent::recursion_guard;
use crate::subagent::types::ForkSubagentConfig;
use crate::message::Message;
use std::sync::Arc;

/// 子代理执行器
#[derive(Clone)]
pub struct SubagentExecutor {
    /// Fork 子代理配置
    fork_config: ForkSubagentConfig,
    /// 是否启用 Coordinator 模式（与 Fork 互斥）
    coordinator_mode: bool,
    /// 是否为非交互式会话（SDK/pipe 模式禁用 Fork）
    non_interactive: bool,
}

impl SubagentExecutor {
    /// 创建执行器
    pub fn new() -> Self {
        Self {
            fork_config: ForkSubagentConfig::default(),
            coordinator_mode: false,
            non_interactive: false,
        }
    }
    
    /// 设置 Fork 配置
    pub fn with_fork_config(mut self, config: ForkSubagentConfig) -> Self {
        self.fork_config = config;
        self
    }
    
    /// 设置 Coordinator 模式状态
    pub fn with_coordinator_mode(mut self, enabled: bool) -> Self {
        self.coordinator_mode = enabled;
        self
    }
    
    /// 设置非交互式会话状态
    pub fn with_non_interactive(mut self, enabled: bool) -> Self {
        self.non_interactive = enabled;
        self
    }
    
    /// 检查 Fork 子代理是否启用
    /// 
    /// 互斥条件：
    /// - Coordinator 模式下禁用
    /// - 非交互式会话（SDK/pipe 模式）禁用
    pub fn is_fork_enabled(&self) -> bool {
        if self.fork_config.enabled {
            if self.coordinator_mode {
                return false;
            }
            if self.non_interactive {
                return false;
            }
            return true;
        }
        false
    }
    
    /// 执行子代理
    pub async fn execute(&self, params: SubagentParams) -> Result<SubagentResult, SubagentError> {
        // 递归防护检查
        match recursion_guard::check_recursion_guard(
            None, // query_source
            &params.prompt_messages,
            &self.fork_config,
        ) {
            recursion_guard::RecursionGuardResult::Deny(reason) => {
                return Err(SubagentError::RecursionGuard(reason));
            }
            recursion_guard::RecursionGuardResult::Allow => {}
        }
        
        // 根据子代理类型选择执行路径
        match params.subagent_type {
            SubagentType::Fork => {
                self.execute_fork(params).await
            }
            SubagentType::GeneralPurpose => {
                self.execute_general(params).await
            }
            SubagentType::Custom(_) => {
                self.execute_custom(params).await
            }
        }
    }
    
    /// 执行 Fork 子代理
    async fn execute_fork(&self, params: SubagentParams) -> Result<SubagentResult, SubagentError> {
        // 构建继承的消息
        let inherited_messages = context_inheritance::build_inherited_messages(
            &params.cache_safe_params.fork_context_messages,
            None,
        );
        
        // 构建 User 消息（包含占位符和指令）
        let tool_use_ids = vec![]; // 从父级 assistant 消息提取
        let user_message = context_inheritance::build_user_message_with_placeholder(
            &self.fork_config.placeholder_result,
            &params.directive,
            &tool_use_ids,
            &self.fork_config,
        );
        
        // 合并消息
        let mut final_messages = inherited_messages;
        final_messages.push(user_message);
        
        // 添加工作树隔离通知（如果有 worktree）
        if let (Some(parent_cwd), Some(worktree_cwd)) = (params.parent_cwd.clone(), params.worktree_path.clone()) {
            let notice = recursion_guard::build_worktree_notice(&parent_cwd, &worktree_cwd);
            final_messages.push(Message::User(crate::message::UserMessage {
                content: vec![crate::message::ContentBlock::Text { text: notice }],
            }));
        }
        
        // 执行子代理查询循环
        // TODO: 集成到实际的 query 循环
        self.run_subagent_loop(final_messages, params).await
    }
    
    /// 执行通用子代理
    async fn execute_general(&self, params: SubagentParams) -> Result<SubagentResult, SubagentError> {
        // 通用子代理从零开始，不继承上下文
        let messages = params.prompt_messages;
        self.run_subagent_loop(messages, SubagentParams::default()).await
    }
    
    /// 执行自定义子代理
    async fn execute_custom(&self, params: SubagentParams) -> Result<SubagentResult, SubagentError> {
        // 自定义子代理执行逻辑
        let messages = params.prompt_messages;
        self.run_subagent_loop(messages, SubagentParams::default()).await
    }
    
    /// 运行子代理查询循环
    async fn run_subagent_loop(
        &self,
        messages: Vec<Message>,
        params: SubagentParams,
    ) -> Result<SubagentResult, SubagentError> {
        // TODO: 实际的查询循环实现
        // 这里返回一个模拟结果
        Ok(SubagentResult {
            messages,
            total_usage: Usage::default(),
            success: true,
        })
    }
}

impl Default for SubagentExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 子代理错误
#[derive(Debug, thiserror::Error)]
pub enum SubagentError {
    #[error("递归防护：{0}")]
    RecursionGuard(String),
    
    #[error("子代理执行失败：{0}")]
    ExecutionFailed(String),
    
    #[error("上下文继承失败：{0}")]
    ContextInheritanceError(String),
    
    #[error("工具集解析失败：{0}")]
    ToolResolutionError(String),
    
    #[error("权限检查失败：{0}")]
    PermissionError(String),
    
    #[error("超时：{0}秒")]
    Timeout(u64),
    
    #[error("内部错误：{0}")]
    InternalError(String),
}
