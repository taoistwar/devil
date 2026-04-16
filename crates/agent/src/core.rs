//! Agent 核心模块
//! 
//! 实现 Agent 的核心逻辑，包括：
//! - Agent 结构体
//! - AgentLoop 异步生成器
//! - 状态管理

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::AgentConfig;
use crate::message::{Message, AssistantMessage, SystemMessage, SystemMessageLevel, ToolUseSummaryMessage};
use crate::state::{State, Terminal, TerminalReason, Continue, ContinueReason};
use crate::deps::{QueryDeps, ProductionDeps, ModelCallParams};
use crate::tools::{ToolRegistry, ToolContext};
use crate::tools::partition::{ConcurrentPartitioner, ToolUseCallInfo};
use crate::tools::executor::{StreamingToolExecutor, BatchToolExecutor, ExecutorConfig, ToolExecutionResult};
use crate::context::{ContextManager, ContextPipelineResult};

/// Agent 状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 暂停中
    Paused,
    /// 已停止
    Stopped,
}

/// 核心 Agent 结构
/// 
/// 整合 channels、memory、plugins 提供完整的 Agent 实现
pub struct Agent {
    /// 配置
    config: AgentConfig,
    /// 当前状态
    status: Arc<RwLock<AgentStatus>>,
    /// 工具注册表
    tool_registry: Arc<RwLock<ToolRegistry>>,
    /// 依赖实现
    deps: Arc<dyn QueryDeps>,
    /// 上下文管理器
    context_manager: ContextManager,
}

impl Agent {
    /// 创建新的 Agent 实例
    pub fn new(config: AgentConfig) -> Result<Self> {
        Ok(Self {
            config,
            status: Arc::new(RwLock::new(AgentStatus::Initializing)),
            tool_registry: Arc::new(RwLock::new(ToolRegistry::new())),
            deps: Arc::new(ProductionDeps::new()),
            context_manager: ContextManager::with_defaults(),
        })
    }

    /// 使用自定义依赖创建 Agent（用于测试）
    pub fn with_deps(config: AgentConfig, deps: Arc<dyn QueryDeps>) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(AgentStatus::Initializing)),
            tool_registry: Arc::new(RwLock::new(ToolRegistry::new())),
            deps,
            context_manager: ContextManager::with_defaults(),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// 获取当前状态
    pub async fn get_status(&self) -> AgentStatus {
        *self.status.read().await
    }

    /// 初始化 Agent
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing agent: {}", self.config.name);
        *self.status.write().await = AgentStatus::Running;
        info!("Agent {} is now running", self.config.name);
        Ok(())
    }

    /// 暂停 Agent
    pub async fn pause(&self) {
        *self.status.write().await = AgentStatus::Paused;
        debug!("Agent {} paused", self.config.name);
    }

    /// 恢复 Agent
    pub async fn resume(&self) {
        *self.status.write().await = AgentStatus::Running;
        debug!("Agent {} resumed", self.config.name);
    }

    /// 停止 Agent
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down agent: {}", self.config.name);
        *self.status.write().await = AgentStatus::Stopped;
        Ok(())
    }

    /// 注册工具
    pub async fn register_tool<T: crate::tools::Tool + 'static>(&self, tool: T) -> Result<()> {
        let mut registry = self.tool_registry.write().await;
        registry.register(tool)?;
        Ok(())
    }

    /// 创建对话循环
    pub fn create_loop(&self, initial_messages: Vec<Message>) -> AgentLoop {
        AgentLoop::new(
            self.config.clone(),
            initial_messages,
            self.deps.clone(),
            self.tool_registry.clone(),
        )
    }

    /// 执行单次对话
    pub async fn run_once(&self, user_message: Message) -> Result<RunResult> {
        let mut loop_instance = self.create_loop(vec![user_message]);
        loop_instance.run().await
    }
}

/// 单次运行结果
#[derive(Debug)]
pub struct RunResult {
    /// 终止状态
    pub terminal: Terminal,
    /// 最终消息历史
    pub messages: Vec<Message>,
    /// 总轮次数
    pub turn_count: usize,
}

/// Agent 对话循环
/// 
/// 基于异步生成器模式实现，支持：
/// - 流式输出
/// - 可取消性
/// - 背压控制
pub struct AgentLoop {
    /// 配置
    config: AgentConfig,
    /// 当前状态
    state: State,
    /// 依赖实现
    deps: Arc<dyn QueryDeps>,
    /// 工具注册表
    tool_registry: Arc<RwLock<ToolRegistry>>,
    /// 工具执行器
    tool_executor: ToolExecutor,
}

impl AgentLoop {
    /// 创建新的对话循环
    pub fn new(
        config: AgentConfig,
        initial_messages: Vec<Message>,
        deps: Arc<dyn QueryDeps>,
        tool_registry: Arc<RwLock<ToolRegistry>>,
    ) -> Self {
        Self {
            config,
            state: State::initial(initial_messages),
            deps,
            tool_registry,
            tool_executor: ToolExecutor::default(),
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> &State {
        &self.state
    }

    /// 获取当前轮次
    pub fn turn_count(&self) -> usize {
        self.state.turn_count
    }

    /// 运行对话循环
    /// 
    /// 这是对话循环的主入口，执行完整的 `while(true)` 循环
    pub async fn run(&mut self) -> Result<RunResult> {
        info!("Starting agent loop for {}", self.config.name);

        // `while(true)` 无限循环 - 每次迭代代表一次"模型调用 + 工具执行"的完整回合
        loop {
            // 检查最大轮数
            if self.state.turn_count >= self.config.max_turns {
                return Ok(RunResult {
                    terminal: Terminal::with_message(
                        TerminalReason::MaxTurns,
                        format!("Reached maximum turns ({})", self.config.max_turns),
                    ),
                    messages: self.state.messages.clone(),
                    turn_count: self.state.turn_count,
                });
            }

            // ===== 阶段一：状态初始化 =====
            // 从状态对象解构当前迭代所需的变量
            let current_turn = self.state.turn_count;
            let messages = self.state.messages.clone();
            
            debug!("Starting turn {} with {} messages", current_turn, messages.len());

            // ===== 阶段二：上下文预处理 =====
            // 执行七步压缩管线
            let pipeline_result = self.context_manager.process_full_pipeline(
                messages,
                &self.config.system_prompt,
                self.config.max_context_tokens,
            ).await?;

            let (processed_messages, system_prompt, token_count) = match pipeline_result {
                ContextPipelineResult::Success { messages, system_prompt, token_count } => {
                    (messages, system_prompt, token_count)
                }
                ContextPipelineResult::TokenLimitExceeded { current_tokens, max_tokens } => {
                    return Ok(RunResult {
                        terminal: Terminal::with_message(
                            TerminalReason::BlockingLimit,
                            format!("Token limit exceeded: {} > {}", current_tokens, max_tokens),
                        ),
                        messages: self.state.messages.clone(),
                        turn_count: self.state.turn_count,
                    });
                }
            };

            debug!("Context preprocessed: {} tokens", token_count);

            // ===== 阶段三：API 调用 =====
            // 调用模型 API
            let call_result = {
                // 检查暂停状态
                // TODO: 需要在外部检查状态，这里简化处理
                
                let params = ModelCallParams {
                    system_prompt,
                    messages: processed_messages,
                    max_tokens: self.config.max_context_tokens / 10,
                    model: self.config.model.clone(),
                };

                match self.deps.call_model(params).await {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Model call failed: {}", e);
                        return Ok(RunResult {
                            terminal: Terminal::with_message(
                                TerminalReason::ModelError,
                                e.to_string(),
                            ),
                            messages: self.state.messages.clone(),
                            turn_count: self.state.turn_count,
                        });
                    }
                }
            };

            debug!(
                "Model response received: {} input tokens, {} output tokens",
                call_result.input_tokens,
                call_result.output_tokens,
            );

            // 将助手消息添加到历史
            self.state.messages.push(Message::Assistant(
                call_result.assistant_message.clone()
            ));

            // ===== 阶段四：工具调用检测与执行 =====
            // 检查是否有工具调用
            let tool_use_blocks = call_result.assistant_message.tool_use_blocks();
            
            if tool_use_blocks.is_empty() {
                // 没有工具调用，进入终止路径
                info!("No tool calls detected, completing turn");
                return Ok(RunResult {
                    terminal: Terminal::new(TerminalReason::Completed),
                    messages: self.state.messages.clone(),
                    turn_count: self.state.turn_count,
                });
            }

            debug!("Detected {} tool call(s)", tool_use_blocks.len());

            // TODO: 执行工具调用
            // 这里简化处理，实际应该：
            // 1. 根据是否启用流式执行选择执行策略
            // 2. 执行权限检查
            // 3. 并发分区调度
            // 4. 收集结果并回填

            // ===== 阶段五：工具结果回填与下一轮 =====
            // 构造新的状态对象，continue 到下一轮
            self.state = self.state.next(
                ContinueReason::NextTurn,
                self.state.messages.clone(),
            );

            info!("Continuing to next turn");
        }
    }

    /// 执行上下文压缩
    async fn execute_auto_compact(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
        let result = self.deps.auto_compact(messages).await?;
        Ok(result.messages)
    }

    /// 执行轻量压缩
    async fn execute_micro_compact(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
        let result = self.deps.micro_compact(messages).await?;
        Ok(result.messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deps::TestDeps;
    use crate::message::UserMessage;

    #[tokio::test]
    async fn test_agent_creation() {
        let config = AgentConfig::default();
        let agent = Agent::new(config).unwrap();
        
        assert_eq!(agent.get_status().await, AgentStatus::Initializing);
        
        agent.initialize().await.unwrap();
        assert_eq!(agent.get_status().await, AgentStatus::Running);
        
        agent.shutdown().await.unwrap();
        assert_eq!(agent.get_status().await, AgentStatus::Stopped);
    }

    #[tokio::test]
    async fn test_agent_with_test_deps() {
        let config = AgentConfig::default();
        let test_deps = Arc::new(TestDeps::empty());
        let agent = Agent::with_deps(config, test_deps);
        
        assert!(agent.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_agent_lifecycle() {
        let config = AgentConfig::default();
        let agent = Agent::new(config).unwrap();
        
        agent.initialize().await.unwrap();
        assert_eq!(agent.get_status().await, AgentStatus::Running);
        
        agent.pause().await;
        assert_eq!(agent.get_status().await, AgentStatus::Paused);
        
        agent.resume().await;
        assert_eq!(agent.get_status().await, AgentStatus::Running);
        
        agent.shutdown().await.unwrap();
        assert_eq!(agent.get_status().await, AgentStatus::Stopped);
    }
}
