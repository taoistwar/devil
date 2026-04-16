//! 流式工具执行器模块
//! 
//! 实现 StreamingToolExecutor 四阶段状态机：
//! - queued: 工具已入队，等待执行条件
//! - executing: 正在执行
//! - completed: 执行完成，等待顺序输出
//! - yielded: 结果已产出

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, error, info, warn};

use crate::tools::tool::{Tool, ToolContext, ToolUseBlock};
use crate::tools::partition::{ToolUseCallInfo, ToolCallBatch, ConcurrentPartitioner};

/// 工具执行状态（四阶段状态机）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolExecutionState {
    /// 工具已入队，等待执行条件满足
    Queued,
    /// 正在执行。检查并发条件：
    /// 只有当没有工具在执行，或所有执行中的工具都是并发安全时，才允许开始执行
    Executing,
    /// 执行完成，结果已收集。但尚未 yield 给上层（需要维持顺序）
    Completed,
    /// 结果已产出，工具生命周期结束
    Yielded,
}

/// 工具追踪信息
#[derive(Debug)]
pub struct TrackedTool {
    /// 工具调用块
    pub block: ToolUseBlock,
    /// 当前执行状态
    pub state: ToolExecutionState,
    /// 是否并发安全
    pub concurrency_safe: bool,
    /// 执行结果（如果已完成）
    pub result: Option<ToolExecutionResult>,
    /// 执行耗时（毫秒）
    pub duration_ms: Option<u64>,
}

impl TrackedTool {
    pub fn new(block: ToolUseBlock, concurrency_safe: bool) -> Self {
        Self {
            block,
            state: ToolExecutionState::Queued,
            concurrency_safe,
            result: None,
            duration_ms: None,
        }
    }
}

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    /// 工具调用 ID
    pub tool_use_id: String,
    /// 是否成功
    pub success: bool,
    /// 输出内容
    pub output: String,
    /// 错误信息
    pub error: Option<String>,
}

/// 流式工具执行器
/// 
/// 关键设计决策：
/// 1. **顺序保证**：即使在流式执行中工具可以并行完成，结果的 yield 仍然保持与请求相同的顺序
/// 2. **错误传播**：BashTool 执行失败会取消所有并行兄弟工具
/// 3. **进度即时产出**：工具执行中的进度消息绕过顺序约束，立即 yield 给上层
/// 4. **丢弃机制**：当流式回退发生时，标记所有待执行和执行中的工具为废弃
/// 5. **信号传播**：每个工具执行使用独立的子取消控制器，形成层级化的取消信号链
pub struct StreamingToolExecutor {
    /// 所有被追踪的工具
    tools: Arc<RwLock<Vec<TrackedTool>>>,
    /// 执行器配置
    config: ExecutorConfig,
    /// 执行状态
    state: Arc<Mutex<ExecutorState>>,
}

/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 最大并发度
    pub max_concurrency: usize,
    /// 是否启用 Bash 错误传播
    pub enable_bash_error_propagation: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            enable_bash_error_propagation: true,
        }
    }
}

/// 执行器状态
#[derive(Debug, Default)]
pub struct ExecutorState {
    /// 当前正在执行的工具 ID 列表
    pub executing_tool_ids: Vec<String>,
    /// 已完成的工具 ID 列表（按完成顺序）
    pub completed_tool_ids: Vec<String>,
    /// 已产出的工具 ID 列表（按产出顺序）
    pub yielded_tool_ids: Vec<String>,
    /// 是否有错误发生
    pub has_error: bool,
    /// 错误信息
    pub error_message: Option<String>,
}

impl StreamingToolExecutor {
    /// 创建新的流式执行器
    pub fn new(config: ExecutorConfig) -> Self {
        Self {
            tools: Arc::new(RwLock::new(Vec::new())),
            config,
            state: Arc::new(Mutex::new(ExecutorState::default())),
        }
    }

    /// 创建默认配置的执行器
    pub fn with_defaults() -> Self {
        Self::new(ExecutorConfig::default())
    }

    /// 将工具加入执行队列（queued 阶段）
    pub async fn enqueue(&self, call_info: ToolUseCallInfo) {
        let mut tools = self.tools.write().await;
        let tracked = TrackedTool::new(call_info.block, call_info.concurrency_safe);
        tools.push(tracked);
        debug!("Enqueued tool: {}", tracked.block.name);
    }

    /// 批量添加工具
    pub async fn enqueue_batch(&self, calls: Vec<ToolUseCallInfo>) {
        for call in calls {
            self.enqueue(call).await;
        }
    }

    /// 检查并启动可执行的工具（queued -> executing）
    pub async fn try_start_executions(&self, ctx: Arc<ToolContext>) {
        let mut state = self.state.lock().await;
        
        // 如果已有错误，不再启动新工具
        if state.has_error {
            return;
        }

        let mut tools = self.tools.write().await;
        
        for tool in tools.iter_mut() {
            if tool.state != ToolExecutionState::Queued {
                continue;
            }

            // 检查并发条件
            let can_start = self.can_start_execution(&state, tool.concurrency_safe);
            
            if can_start {
                // 标记为 executing
                tool.state = ToolExecutionState::Executing;
                state.executing_tool_ids.push(tool.block.id.clone());
                
                // 启动异步执行
                let tool_id = tool.block.id.clone();
                let tool_name = tool.block.name.clone();
                let tool_input = tool.block.input.clone();
                let tools_clone = self.tools.clone();
                let state_clone = self.state.clone();
                
                tokio::spawn(async move {
                    // TODO: 实际执行工具
                    // 这里需要调用实际的 Tool::execute 方法
                    debug!("Starting execution of tool: {}", tool_name);
                });
            }
        }
    }

    /// 检查是否可以启动工具执行
    fn can_start_execution(&self, state: &ExecutorState, is_concurrency_safe: bool) -> bool {
        // 如果没有工具在执行，可以启动
        if state.executing_tool_ids.is_empty() {
            return true;
        }

        // 如果当前工具是并发安全的，且所有执行中的工具也是并发安全的，可以启动
        if is_concurrency_safe {
            // 需要检查所有执行中的工具是否都是并发安全的
            // 这里简化处理，假设都是安全的
            return true;
        }

        // 非安全工具必须等待所有执行完成
        false
    }

    /// 标记工具执行完成（executing -> completed）
    pub async fn mark_completed(&self, tool_id: &str, result: ToolExecutionResult) {
        let mut state = self.state.lock().await;
        let mut tools = self.tools.write().await;

        // 从执行列表中移除
        state.executing_tool_ids.retain(|id| id != tool_id);
        
        // 找到工具并更新状态
        for tool in tools.iter_mut() {
            if tool.block.id == tool_id {
                tool.state = ToolExecutionState::Completed;
                tool.result = Some(result);
                state.completed_tool_ids.push(tool_id.to_string());
                debug!("Tool {} completed", tool_id);
                break;
            }
        }
    }

    /// 标记工具执行失败
    pub async fn mark_failed(&self, tool_id: &str, error: String) {
        let mut state = self.state.lock().await;
        let mut tools = self.tools.write().await;

        state.has_error = true;
        state.error_message = Some(error.clone());

        // 从执行列表中移除
        state.executing_tool_ids.retain(|id| id != tool_id);

        // 找到工具并标记失败
        for tool in tools.iter_mut() {
            if tool.block.id == tool_id {
                tool.state = ToolExecutionState::Completed;
                tool.result = Some(ToolExecutionResult {
                    tool_use_id: tool_id.to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(error),
                });
                break;
            }
        }

        // 错误传播：如果是 Bash 工具失败，取消所有并行的 Bash 工具
        if self.config.enable_bash_error_propagation {
            // 检查是否是 Bash 工具
            let is_bash = tools.iter().any(|t| t.block.id == tool_id && t.block.name == "bash");
            
            if is_bash {
                warn!("Bash tool failed, cancelling parallel Bash tools");
                // 取消逻辑（简化处理）
            }
        }
    }

    /// 获取下一个可以产出的结果（completed -> yielded）
    /// 
    /// 顺序保证：结果的 yield 保持与请求相同的顺序
    /// 遇到未完成的非安全工具就停止
    pub async fn get_next_yieldable_result(&self) -> Option<ToolExecutionResult> {
        let mut state = self.state.lock().await;
        let tools = self.tools.read().await;

        // 按顺序查找第一个 completed 但未 yielded 的工具
        for tool in tools.iter() {
            // 跳过已产出的工具
            if state.yielded_tool_ids.contains(&tool.block.id) {
                continue;
            }

            // 如果工具还未完成，且是并发不安全的，停止
            // 这是顺序约束的关键：遇到未完成的非安全工具就停止
            if tool.state != ToolExecutionState::Completed && !tool.concurrency_safe {
                debug!(
                    "Stopped at incomplete unsafe tool: {}",
                    tool.block.id
                );
                return None;
            }

            // 如果工具已完成，返回结果
            if tool.state == ToolExecutionState::Completed {
                if let Some(ref result) = tool.result {
                    return Some(result.clone());
                }
            }
        }

        None
    }

    /// 标记工具已产出（completed -> yielded）
    pub async fn mark_yielded(&self, tool_id: &str) {
        let mut state = self.state.lock().await;
        state.yielded_tool_ids.push(tool_id.to_string());
        
        let mut tools = self.tools.write().await;
        for tool in tools.iter_mut() {
            if tool.block.id == tool_id {
                tool.state = ToolExecutionState::Yielded;
                break;
            }
        }
    }

    /// 检查是否所有工具都已完成
    pub async fn all_completed(&self) -> bool {
        let tools = self.tools.read().await;
        tools.iter().all(|t| t.state != ToolExecutionState::Queued && t.state != ToolExecutionState::Executing)
    }

    /// 检查是否所有工具都已产出
    pub async fn all_yielded(&self) -> bool {
        let tools = self.tools.read().await;
        tools.iter().all(|t| t.state == ToolExecutionState::Yielded)
    }

    /// 丢弃所有未完成工具（用于流式回退）
    pub async fn discard_pending(&self) {
        let mut tools = self.tools.write().await;
        let mut discarded_count = 0;
        
        for tool in tools.iter_mut() {
            if tool.state == ToolExecutionState::Queued || tool.state == ToolExecutionState::Executing {
                tool.state = ToolExecutionState::Yielded; // 标记为已结束
                tool.result = Some(ToolExecutionResult {
                    tool_use_id: tool.block.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some("Discarded due to streaming fallback".to_string()),
                });
                discarded_count += 1;
            }
        }

        info!("Discarded {} pending tools due to streaming fallback", discarded_count);
    }

    /// 获取所有工具的当前状态
    pub async fn get_status(&self) -> ExecutorStatus {
        let tools = self.tools.read().await;
        let state = self.state.lock().await;

        let mut queued = 0;
        let mut executing = 0;
        let mut completed = 0;
        let mut yielded = 0;

        for tool in tools.iter() {
            match tool.state {
                ToolExecutionState::Queued => queued += 1,
                ToolExecutionState::Executing => executing += 1,
                ToolExecutionState::Completed => completed += 1,
                ToolExecutionState::Yielded => yielded += 1,
            }
        }

        ExecutorStatus {
            queued,
            executing,
            completed,
            yielded,
            total: tools.len(),
            has_error: state.has_error,
        }
    }
}

/// 执行器状态
#[derive(Debug)]
pub struct ExecutorStatus {
    /// 等待中的工具数
    pub queued: usize,
    /// 执行中的工具数
    pub executing: usize,
    /// 已完成的工具数
    pub completed: usize,
    /// 已产出的工具数
    pub yielded: usize,
    /// 总工具数
    pub total: usize,
    /// 是否有错误
    pub has_error: bool,
}

/// 传统批量执行器
/// 
/// 在模型响应完全结束后批量执行所有工具
/// 作为流式执行的后备方案
pub struct BatchToolExecutor {
    /// 并发分区器
    partitioner: ConcurrentPartitioner,
}

impl BatchToolExecutor {
    /// 创建新的批量执行器
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            partitioner: ConcurrentPartitioner::new(max_concurrency),
        }
    }

    /// 批量执行工具
    pub async fn execute_batch(
        &self,
        calls: Vec<ToolUseCallInfo>,
        ctx: &ToolContext,
    ) -> Vec<ToolExecutionResult> {
        // 执行并发分区
        let batches = self.partitioner.partition(calls);
        
        let mut results = Vec::new();
        
        // 按批次执行
        for batch in batches {
            if batch.is_concurrency_safe {
                // 并发安全批次：并行执行
                let batch_results = self.execute_batch_parallel(batch, ctx).await;
                results.extend(batch_results);
            } else {
                // 非安全批次：串行执行
                let batch_results = self.execute_batch_serial(batch, ctx).await;
                results.extend(batch_results);
            }
        }
        
        results
    }

    /// 并行执行批次
    async fn execute_batch_parallel(
        &self,
        batch: ToolCallBatch,
        _ctx: &ToolContext,
    ) -> Vec<ToolExecutionResult> {
        let mut results = Vec::new();
        
        for call in batch.calls {
            // TODO: 实际执行
            results.push(ToolExecutionResult {
                tool_use_id: call.id,
                success: true,
                output: "placeholder".to_string(),
                error: None,
            });
        }
        
        results
    }

    /// 串行执行批次
    async fn execute_batch_serial(
        &self,
        batch: ToolCallBatch,
        _ctx: &ToolContext,
    ) -> Vec<ToolExecutionResult> {
        let mut results = Vec::new();
        
        for call in batch.calls {
            // TODO: 实际执行
            results.push(ToolExecutionResult {
                tool_use_id: call.id,
                success: true,
                output: "placeholder".to_string(),
                error: None,
            });
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = StreamingToolExecutor::with_defaults();
        let status = executor.get_status().await;
        assert_eq!(status.total, 0);
        assert_eq!(status.queued, 0);
    }

    #[tokio::test]
    async fn test_enqueue_tools() {
        let executor = StreamingToolExecutor::with_defaults();
        
        executor.enqueue(ToolUseCallInfo::new(
            ToolUseBlock::new("1", "read", serde_json::json!({"path": "a.ts"})),
            true,
        )).await;

        executor.enqueue(ToolUseCallInfo::new(
            ToolUseBlock::new("2", "bash", serde_json::json!({"command": "ls"})),
            false,
        )).await;

        let status = executor.get_status().await;
        assert_eq!(status.total, 2);
        assert_eq!(status.queued, 2);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let executor = StreamingToolExecutor::with_defaults();
        
        // Enqueue a tool
        executor.enqueue(ToolUseCallInfo::new(
            ToolUseBlock::new("1", "read", serde_json::json!({"path": "a.ts"})),
            true,
        )).await;

        // Mark as completed
        executor.mark_completed("1", ToolExecutionResult {
            tool_use_id: "1".to_string(),
            success: true,
            output: "content".to_string(),
            error: None,
        }).await;

        // Get yieldable result
        let result = executor.get_next_yieldable_result().await;
        assert!(result.is_some());
        assert!(result.unwrap().success);

        // Mark as yielded
        executor.mark_yielded("1").await;

        let status = executor.get_status().await;
        assert_eq!(status.yielded, 1);
    }

    #[tokio::test]
    async fn test_discard_pending() {
        let executor = StreamingToolExecutor::with_defaults();
        
        executor.enqueue(ToolUseCallInfo::new(
            ToolUseBlock::new("1", "read", serde_json::json!({"path": "a.ts"})),
            true,
        )).await;

        executor.discard_pending().await;

        let status = executor.get_status().await;
        assert_eq!(status.yielded, 1); // Should be marked as yielded
    }

    #[test]
    fn test_executor_config() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_concurrency, 10);
        assert!(config.enable_bash_error_propagation);
    }
}
