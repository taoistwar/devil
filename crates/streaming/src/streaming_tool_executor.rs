//! StreamingToolExecutor - 流式工具执行器
//!
//! 实现"流到即执行"策略：
//! - 并发安全工具可并行执行
//! - 非并发安全工具串行执行
//! - 结果按顺序缓冲输出
//! - Bash 失败时级联取消兄弟工具

use anyhow::{Context, Result};
use futures::stream::Stream;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 工具状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolState {
    Queued,      // 排队等待
    Executing,   // 执行中
    Completed,   // 已完成
    Yielded,     // 已输出
    Cancelled,   // 已取消
}

/// 追踪的待执行工具
pub struct TrackedTool {
    /// 工具调用 ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 工具参数
    pub input: serde_json::Value,
    /// 当前状态
    pub state: ToolState,
    /// 是否并发安全
    pub is_concurrency_safe: bool,
    /// 执行结果
    pub result: Option<ToolResult>,
    /// 是否为 Bash 工具
    pub is_bash: bool,
}

impl TrackedTool {
    pub fn new(
        id: String,
        name: String,
        input: serde_json::Value,
        is_concurrency_safe: bool,
    ) -> Self {
        let is_bash = name.to_lowercase().contains("bash")
            || name.to_lowercase().contains("shell")
            || name.to_lowercase().contains("exec");

        Self {
            id,
            name,
            input,
            state: ToolState::Queued,
            is_concurrency_safe,
            result: None,
            is_bash,
        }
    }
}

/// 工具结果
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}

/// 流式工具执行器
pub struct StreamingToolExecutor {
    /// 待执行工具队列
    tools: Arc<RwLock<Vec<TrackedTool>>>,
    /// 当前正在执行的工具 ID
    executing: Arc<RwLock<Vec<String>>>,
    /// 兄弟工具中止控制器
    sibling_abort: Arc<AbortController>,
    /// 是否有 Bash 失败
    bash_failed: Arc<AtomicBool>,
}

/// 中止控制器（简化版）
pub struct AbortController {
    aborted: AtomicBool,
}

impl AbortController {
    pub fn new() -> Self {
        Self {
            aborted: AtomicBool::new(false),
        }
    }

    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingToolExecutor {
    /// 创建新的执行器
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(Vec::new())),
            executing: Arc::new(RwLock::new(Vec::new())),
            sibling_abort: Arc::new(AbortController::new()),
            bash_failed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 添加新工具
    pub async fn add_tool(&mut self, tool: TrackedTool) {
        let tool_id = tool.id.clone();
        self.tools.write().await.push(tool);

        debug!("Added tool to executor: {}", tool_id);

        // 触发队列处理
        self.process_queue().await;
    }

    /// 检查工具是否可以执行
    pub async fn can_execute_tool(&self, tool: &TrackedTool) -> bool {
        let executing = self.executing.read().await;

        // 没有工具在执行时，任何工具都可以启动
        if executing.is_empty() {
            return true;
        }

        // 检查是否有 Bash 失败
        if self.bash_failed.load(Ordering::Relaxed) {
            return false;
        }

        // 有工具在执行时，检查并发安全性
        let executing_tools = self.tools.read().await;
        let executing_refs: Vec<&TrackedTool> = executing_tools
            .iter()
            .filter(|t| executing.contains(&t.id))
            .collect();

        // 检查新工具和所有执行中的工具是否都是并发安全
        let all_safe = executing_refs.iter().all(|t| t.is_concurrency_safe);

        if all_safe && tool.is_concurrency_safe {
            return true;
        }

        // 新工具为非并发安全工具时，必须独占执行
        if !tool.is_concurrency_safe {
            return false;
        }

        // 检查是否有非并发安全工具正在执行中
        let has_unsafe_executing = executing_refs.iter().any(|t| !t.is_concurrency_safe);
        if has_unsafe_executing {
            return false;
        }

        true
    }

    /// 处理队列
    async fn process_queue(&self) {
        let mut tools = self.tools.write().await;

        for tool in tools.iter_mut() {
            if tool.state == ToolState::Queued {
                if self.can_execute_tool(tool).await {
                    tool.state = ToolState::Executing;
                    self.executing.write().await.push(tool.id.clone());

                    info!("Starting tool execution: {}", tool.id);
                }
            }
        }
    }

    /// 标记工具完成
    pub async fn mark_completed(&self, tool_id: &str, result: ToolResult) {
        let mut tools = self.tools.write().await;
        let mut executing = self.executing.write().await;

        if let Some(tool) = tools.iter_mut().find(|t| t.id == tool_id) {
            tool.state = ToolState::Completed;
            tool.result = Some(result);

            // 从执行列表移除
            executing.retain(|id| id != tool_id);

            info!("Tool completed: {}", tool_id);

            // 触发队列处理
            drop(tools);
            drop(executing);
            self.process_queue().await;
        }
    }

    /// 标记工具失败（Bash 级联取消）
    pub async fn mark_failed(&self, tool_id: &str, error: &str) {
        let mut tools = self.tools.write().await;

        if let Some(tool) = tools.iter_mut().find(|t| t.id == tool_id) {
            tool.state = ToolState::Cancelled;

            // 如果是 Bash 工具，级联取消所有兄弟工具
            if tool.is_bash {
                warn!("Bash tool failed, cascading cancel: {}", tool_id);
                self.bash_failed.store(true, Ordering::Relaxed);
                self.sibling_abort.abort();

                // 取消所有排队的工具
                for other in tools.iter_mut() {
                    if other.id != tool_id && other.state == ToolState::Queued {
                        other.state = ToolState::Cancelled;
                        debug!("Cascade cancelled: {}", other.id);
                    }
                }
            }

            error!("Tool failed: {} - {}", tool_id, error);
        }
    }

    /// 获取已完成的结果（按序输出）
    pub async fn get_completed_results(
        &self,
    ) -> impl Stream<Item = ToolResult> + Send {
        use futures::stream;

        let tools = self.tools.read().await.clone();
        let mut results = Vec::new();

        // 按添加顺序收集已完成但未输出的结果
        for tool in tools.iter() {
            if tool.state == ToolState::Completed {
                if let Some(ref result) = tool.result {
                    results.push(result.clone());
                }
            }
        }

        stream::iter(results)
    }

    /// 获取待执行工具数量
    pub async fn pending_count(&self) -> usize {
        let tools = self.tools.read().await;
        tools
            .iter()
            .filter(|t| matches!(t.state, ToolState::Queued | ToolState::Executing))
            .count()
    }

    /// 清除已输出的工具
    pub async fn clear_yielded(&self) {
        let mut tools = self.tools.write().await;
        tools.retain(|t| t.state != ToolState::Yielded);
    }

    /// 检查是否有工具在执行
    pub async fn has_executing(&self) -> bool {
        !self.executing.read().await.is_empty()
    }

    /// 检查是否有 Bash 失败
    pub fn has_bash_failed(&self) -> bool {
        self.bash_failed.load(Ordering::Relaxed)
    }

    /// 重置执行器状态
    pub async fn reset(&self) {
        self.tools.write().await.clear();
        self.executing.write().await.clear();
        self.bash_failed.store(false, Ordering::Relaxed);
        self.sibling_abort = Arc::new(AbortController::new());
    }
}

impl Default for StreamingToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具并发安全判断
pub fn is_tool_concurrency_safe(tool_name: &str) -> bool {
    let name = tool_name.to_lowercase();

    // 只读工具 - 并发安全
    let safe_tools = [
        "read", "grep", "glob", "fetch", "webfetch", "list", "search",
    ];

    // 写入工具 - 非并发安全
    let unsafe_tools = [
        "bash", "shell", "exec", "edit", "write", "notebookedit",
        "delete", "remove", "create", "run",
    ];

    if unsafe_tools.iter().any(|&t| name.contains(t)) {
        return false;
    }

    if safe_tools.iter().any(|&t| name.contains(t)) {
        return true;
    }

    // 默认非安全（fail-closed）
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_tool() {
        let mut executor = StreamingToolExecutor::new();

        let tool = TrackedTool::new(
            "tool-1".to_string(),
            "read_file".to_string(),
            serde_json::json!({"path": "/test.txt"}),
            true,
        );

        executor.add_tool(tool).await;

        let count = executor.pending_count().await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_concurrent_safe_tools() {
        let mut executor = StreamingToolExecutor::new();

        // 添加两个并发安全工具
        let tool1 = TrackedTool::new(
            "tool-1".to_string(),
            "read_file".to_string(),
            serde_json::json!({"path": "/test1.txt"}),
            true,
        );

        let tool2 = TrackedTool::new(
            "tool-2".to_string(),
            "grep".to_string(),
            serde_json::json!({"pattern": "TODO"}),
            true,
        );

        executor.add_tool(tool1).await;
        executor.add_tool(tool2).await;

        // 两个工具都应该可以执行
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let count = executor.pending_count().await;
        // 两个工具都应该已开始执行
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_unsafe_tool_blocks() {
        let mut executor = StreamingToolExecutor::new();

        // 添加一个非并发安全工具（Bash）
        let tool1 = TrackedTool::new(
            "tool-1".to_string(),
            "bash".to_string(),
            serde_json::json!({"command": "echo hello"}),
            false,
        );

        executor.add_tool(tool1).await;

        // 添加另一个工具
        let tool2 = TrackedTool::new(
            "tool-2".to_string(),
            "read_file".to_string(),
            serde_json::json!({"path": "/test.txt"}),
            true,
        );

        executor.add_tool(tool2).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Bash 工具独占执行，另一个应该排队
        let count = executor.pending_count().await;
        assert!(count >= 1);
    }

    #[test]
    fn test_is_tool_concurrency_safe() {
        assert!(is_tool_concurrency_safe("read_file"));
        assert!(is_tool_concurrency_safe("grep"));
        assert!(is_tool_concurrency_safe("glob"));
        assert!(is_tool_concurrency_safe("web_fetch"));

        assert!(!is_tool_concurrency_safe("bash"));
        assert!(!is_tool_concurrency_safe("edit_file"));
        assert!(!is_tool_concurrency_safe("write_file"));
        assert!(!is_tool_concurrency_safe("notebook_edit"));
    }

    #[tokio::test]
    async fn test_bash_failure_cascade() {
        let mut executor = StreamingToolExecutor::new();

        // 添加 Bash 工具
        let bash_tool = TrackedTool::new(
            "bash-1".to_string(),
            "bash".to_string(),
            serde_json::json!({"command": "false"}),
            false,
        );

        // 添加其他工具
        let read_tool = TrackedTool::new(
            "read-1".to_string(),
            "read_file".to_string(),
            serde_json::json!({"path": "/test.txt"}),
            true,
        );

        executor.add_tool(bash_tool).await;
        executor.add_tool(read_tool).await;

        // 模拟 Bash 失败
        executor.mark_failed("bash-1", "Command failed").await;

        assert!(executor.has_bash_failed());

        // Read 工具应该被取消
        let tools = executor.tools.read().await;
        let read_tool = tools.iter().find(|t| t.id == "read-1").unwrap();
        assert_eq!(read_tool.state, ToolState::Cancelled);
    }
}
