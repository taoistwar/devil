//! 任务编排逻辑
//!
//! 实现任务的派发、综合和验证流程

use crate::coordinator::types::{TaskNotification, TaskPhase, TaskStatus, WorkerDirective};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 任务编排器
///
/// 负责任务的派发、结果收集和综合
pub struct Orchestrator {
    /// 运行中的任务列表
    running_tasks: Arc<RwLock<Vec<RunningTask>>>,
}

/// 运行中的任务
#[derive(Debug, Clone)]
pub struct RunningTask {
    /// 任务 ID
    pub task_id: String,
    /// 任务描述
    pub description: String,
    /// 任务阶段
    pub phase: TaskPhase,
    /// Worker 指令
    pub directive: WorkerDirective,
    /// 子 Agent 嵌套深度
    pub depth: u8,
}

/// 最大子 Agent 嵌套深度
pub const MAX_SUBAGENT_DEPTH: u8 = 3;

impl Orchestrator {
    /// 创建编排器
    pub fn new() -> Self {
        Self {
            running_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 派发研究任务（可并行）
    pub async fn spawn_research(
        &self,
        description: impl Into<String>,
        prompt: impl Into<String>,
    ) -> String {
        let directive = WorkerDirective::research(description, prompt);
        self.spawn_task(directive, TaskPhase::Research).await
    }

    /// 派发实现任务
    pub async fn spawn_implementation(
        &self,
        description: impl Into<String>,
        prompt: impl Into<String>,
    ) -> String {
        let directive = WorkerDirective::implement(description, prompt);
        self.spawn_task(directive, TaskPhase::Implementation).await
    }

    /// 派发验证任务
    pub async fn spawn_verification(
        &self,
        description: impl Into<String>,
        prompt: impl Into<String>,
    ) -> String {
        let directive = WorkerDirective::verify(description, prompt);
        self.spawn_task(directive, TaskPhase::Verification).await
    }

    /// 派发任务（通用）
    async fn spawn_task(&self, directive: WorkerDirective, phase: TaskPhase) -> String {
        self.spawn_task_with_depth(directive, phase, 0).await
    }

    /// 派发带深度的任务
    async fn spawn_task_with_depth(
        &self,
        directive: WorkerDirective,
        phase: TaskPhase,
        depth: u8,
    ) -> String {
        let task_id = format!(
            "agent-{}",
            &uuid::Uuid::new_v4().to_string()[..8]
        );

        let running_task = RunningTask {
            task_id: task_id.clone(),
            description: directive.description.clone(),
            phase: phase.clone(),
            directive,
            depth,
        };

        let mut tasks = self.running_tasks.write().await;
        tasks.push(running_task);

        tracing::info!(
            "派发 {} (depth={}, phase={})",
            task_id,
            depth,
            format!("{:?}", phase)
        );

        task_id
    }

    /// 检查是否可以派发子 Agent
    ///
    /// 根据 FR-010 和 SC-004，限制最大嵌套深度为 3 层
    pub fn can_spawn_subagent(&self, parent_depth: u8) -> bool {
        parent_depth < MAX_SUBAGENT_DEPTH
    }

    /// 派发子 Agent（从 Worker 内部调用）
    ///
    /// Worker 可以派发自己的子 Agent 来处理嵌套任务
    pub async fn spawn_subagent(
        &self,
        parent_task_id: &str,
        directive: WorkerDirective,
        phase: TaskPhase,
    ) -> Result<String, String> {
        let mut tasks = self.running_tasks.write().await;

        let parent_depth = tasks
            .iter()
            .find(|t| t.task_id == parent_task_id)
            .map(|t| t.depth)
            .ok_or_else(|| format!("未找到父任务: {}", parent_task_id))?;

        if parent_depth >= MAX_SUBAGENT_DEPTH {
            return Err(format!(
                "已达最大嵌套深度 {}，禁止再派发子 Agent",
                MAX_SUBAGENT_DEPTH
            ));
        }

        let task_id = format!(
            "subagent-{}",
            &uuid::Uuid::new_v4().to_string()[..8]
        );

        let running_task = RunningTask {
            task_id: task_id.clone(),
            description: directive.description.clone(),
            phase,
            directive,
            depth: parent_depth + 1,
        };

        tasks.push(running_task);

        drop(tasks);

        tracing::info!(
            "Worker {} 派发子 Agent {} (depth={})",
            parent_task_id,
            task_id,
            parent_depth + 1
        );

        Ok(task_id)
    }

    /// 继续运行中的任务
    ///
    /// 通过 SendMessage 继续已存在的 worker
    pub async fn continue_task(
        &self,
        task_id: &str,
        message: impl Into<String>,
    ) -> Result<(), String> {
        let tasks = self.running_tasks.read().await;
        let found = tasks.iter().any(|t| t.task_id == task_id);

        if !found {
            return Err(format!("未找到任务：{}", task_id));
        }

        let msg = message.into();
        tracing::info!("继续任务 {}: {}", task_id, msg);

        Ok(())
    }

    /// 派发 Worker Agent
    ///
    /// 创建 SubagentParams 并执行 worker
    pub async fn spawn_worker(
        &self,
        directive: WorkerDirective,
    ) -> Result<String, String> {
        let task_id = format!("agent-{}", &uuid::Uuid::new_v4().to_string()[..8]);

        let running_task = RunningTask {
            task_id: task_id.clone(),
            description: directive.description.clone(),
            phase: TaskPhase::Implementation,
            directive: directive.clone(),
            depth: 0,
        };

        {
            let mut tasks = self.running_tasks.write().await;
            tasks.push(running_task);
        }

        tracing::info!(
            "派发 Worker {}: {}",
            task_id,
            directive.description
        );

        Ok(task_id)
    }

    /// 停止运行中的任务
    pub async fn stop_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.running_tasks.write().await;
        let index = tasks.iter().position(|t| t.task_id == task_id);

        match index {
            Some(idx) => {
                tasks.remove(idx);
                Ok(())
            }
            None => Err(format!("未找到任务：{}", task_id)),
        }
    }

    /// 停止超时任务
    ///
    /// 根据 SC-005，Worker 失败需在 10 秒内检测
    pub async fn stop_timeout_tasks(&self, _timeout_seconds: u64) -> Vec<String> {
        let timed_out = Vec::new();
        let mut tasks = self.running_tasks.write().await;

        // 保留未超时的任务，返回超时的任务 ID
        tasks.retain(|_t| {
            // TODO: 实现实际超时检测（需要记录任务开始时间）
            // 这里暂时返回空列表
            true
        });

        timed_out
    }

    /// 处理任务完成通知
    pub async fn on_task_completed(&self, notification: TaskNotification) {
        let mut tasks = self.running_tasks.write().await;
        tasks.retain(|t| t.task_id != notification.task_id);
    }

    /// 处理任务失败通知
    pub async fn on_task_failed(&self, notification: TaskNotification) -> TaskFailureAction {
        let mut tasks = self.running_tasks.write().await;
        let failed_task = tasks.iter().find(|t| t.task_id == notification.task_id);

        let action = if let Some(task) = failed_task {
            TaskFailureAction::ReportToUser {
                task_id: notification.task_id.clone(),
                description: task.description.clone(),
                error: notification.summary.clone(),
            }
        } else {
            TaskFailureAction::UnknownTask
        };

        tasks.retain(|t| t.task_id != notification.task_id);
        action
    }

    /// 聚合多个 Worker 的结果
    ///
    /// 根据 Claude Code 协调器模式，结果以 `<task-notification>` XML 格式返回
    pub async fn aggregate_results(&self, notifications: Vec<TaskNotification>) -> AggregatedResults {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut killed = Vec::new();

        for notification in notifications {
            match notification.status {
                TaskStatus::Completed => successful.push(notification),
                TaskStatus::Failed => failed.push(notification),
                TaskStatus::Killed => killed.push(notification),
            }
        }

        let total_count = successful.len() + failed.len() + killed.len();

        AggregatedResults {
            successful,
            failed,
            killed,
            total_count,
        }
    }

    /// 综合多个研究结果
    pub async fn synthesize_research(&self, findings: Vec<TaskNotification>) -> String {
        let mut summary = String::new();

        for finding in findings {
            if finding.status == TaskStatus::Completed {
                if let Some(result) = finding.result {
                    summary.push_str(&format!("任务发现：{}\n\n", result));
                }
            }
        }

        summary
    }

    /// 重新派发失败的任务
    pub async fn reassign_task(&self, directive: WorkerDirective) -> Result<String, String> {
        self.spawn_worker(directive).await
    }

    /// 选择继续还是新派发任务
    ///
    /// 根据上下文重叠度决定：
    /// - 高重叠：继续（SendMessage）
    /// - 低重叠：新派发（Agent）
    pub fn should_continue_or_spawn(
        &self,
        previous_task: Option<&RunningTask>,
        new_purpose: &str,
    ) -> ContinueOrSpawn {
        match previous_task {
            None => ContinueOrSpawn::Spawn, // 没有之前的任务，派发新的

            Some(task) => {
                // 根据任务阶段和目的决定
                match (task.phase.clone(), new_purpose) {
                    // 研究完成后，如果是实现相同内容，继续
                    (TaskPhase::Research, p) if p.contains("implement") => {
                        ContinueOrSpawn::Continue
                    }
                    // 实现完成后，验证同一内容，派发新的（新鲜视角）
                    (TaskPhase::Implementation, p) if p.contains("verify") => {
                        ContinueOrSpawn::Spawn
                    }
                    // 纠正失败，继续（有错误上下文）
                    (_, p) if p.contains("fix") || p.contains("retry") => ContinueOrSpawn::Continue,
                    // 完全无关的任务，派发新的
                    _ => ContinueOrSpawn::Spawn,
                }
            }
        }
    }

    /// 获取运行中的任务数量
    pub async fn running_count(&self) -> usize {
        let tasks = self.running_tasks.read().await;
        tasks.len()
    }

    /// 获取所有运行中的任务
    pub async fn get_running_tasks(&self) -> Vec<RunningTask> {
        let tasks = self.running_tasks.read().await;
        tasks.clone()
    }

    /// 获取协调器状态
    pub async fn get_coordinator_status(&self, enabled: bool, simple_mode: bool) -> crate::coordinator::types::CoordinatorStatus {
        let tasks = self.running_tasks.read().await;
        let active_workers = tasks
            .iter()
            .map(|t| crate::coordinator::types::WorkerStatus {
                task_id: t.task_id.clone(),
                description: t.description.clone(),
                phase: format!("{:?}", t.phase),
            })
            .collect();

        crate::coordinator::types::CoordinatorStatus {
            enabled,
            active_workers,
            simple_mode,
            total_workers_spawned: tasks.len(),
        }
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// 继续或派发决策
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinueOrSpawn {
    /// 继续现有任务（SendMessage）
    Continue,
    /// 派发新任务（Agent）
    Spawn,
}

/// 任务失败处理动作
#[derive(Debug, Clone)]
pub enum TaskFailureAction {
    /// 报告给用户
    ReportToUser {
        task_id: String,
        description: String,
        error: String,
    },
    /// 重新派发任务
    Reassign {
        original_task_id: String,
        reason: String,
    },
    /// 未知任务
    UnknownTask,
}

/// 聚合结果
#[derive(Debug, Clone)]
pub struct AggregatedResults {
    /// 成功的任务
    pub successful: Vec<TaskNotification>,
    /// 失败的任务
    pub failed: Vec<TaskNotification>,
    /// 被停止的任务
    pub killed: Vec<TaskNotification>,
    /// 总数
    pub total_count: usize,
}

impl AggregatedResults {
    /// 检查是否所有任务都成功
    pub fn all_succeeded(&self) -> bool {
        self.failed.is_empty() && self.killed.is_empty()
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            return 1.0;
        }
        self.successful.len() as f64 / self.total_count as f64
    }

    /// 生成用户友好的摘要
    pub fn summary(&self) -> String {
        if self.all_succeeded() {
            format!(
                "所有 {} 个任务已完成",
                self.total_count
            )
        } else {
            format!(
                "完成 {}/{} 个任务 (成功: {}, 失败: {}, 停止: {})",
                self.successful.len(),
                self.total_count,
                self.successful.len(),
                self.failed.len(),
                self.killed.len()
            )
        }
    }
}

/// 构建 Worker 提示词的最佳实践
pub struct PromptBuilder {
    description: String,
    prompt: String,
    purpose: Option<String>,
}

impl PromptBuilder {
    /// 创建提示词构建器
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            prompt: String::new(),
            purpose: None,
        }
    }

    /// 添加具体的文件路径和行号
    pub fn with_file_location(
        mut self,
        file_path: &str,
        line_number: Option<u32>,
        context: &str,
    ) -> Self {
        let location = match line_number {
            Some(line) => format!("{} (第{}行)", file_path, line),
            None => file_path.to_string(),
        };

        self.prompt
            .push_str(&format!("在 {} 中，{}。", location, context));

        self
    }

    /// 添加目的说明
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// 添加具体操作指令
    pub fn with_action(mut self, action: &str) -> Self {
        self.prompt.push_str(action);
        self.prompt.push('\n');

        self
    }

    /// 添加报告要求
    pub fn with_report_requirement(mut self, requirement: &str) -> Self {
        self.prompt
            .push_str(&format!("报告要求：{}\n", requirement));

        self
    }

    /// 构建 Worker 指令
    pub fn build(self) -> WorkerDirective {
        WorkerDirective {
            description: self.description,
            prompt: self.prompt,
            purpose: self.purpose,
            subagent_type: "worker".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_spawn() {
        let orchestrator = Orchestrator::new();

        let task_id = orchestrator
            .spawn_research("调查 auth bug", "研究问题".to_string())
            .await;

        assert!(task_id.starts_with("agent-"));
        assert_eq!(orchestrator.running_count().await, 1);
    }

    #[tokio::test]
    async fn test_orchestrator_stop() {
        let orchestrator = Orchestrator::new();

        let task_id = orchestrator
            .spawn_research("调查 auth bug", "研究问题".to_string())
            .await;

        orchestrator.stop_task(&task_id).await.unwrap();
        assert_eq!(orchestrator.running_count().await, 0);
    }

    #[test]
    fn test_continue_or_spawn_decision() {
        let orchestrator = Orchestrator::new();

        let research_task = Some(RunningTask {
            task_id: "agent-123".to_string(),
            description: "Research".to_string(),
            phase: TaskPhase::Research,
            directive: WorkerDirective::research("desc", "prompt"),
        });

        // 研究完成后实现，继续
        assert_eq!(
            orchestrator.should_continue_or_spawn(research_task.as_ref(), "implement the fix"),
            ContinueOrSpawn::Continue
        );

        // 实现完成后验证，派发新的
        let impl_task = Some(RunningTask {
            task_id: "agent-123".to_string(),
            description: "Implement".to_string(),
            phase: TaskPhase::Implementation,
            directive: WorkerDirective::implement("desc", "prompt"),
            depth: 0,
        });

        assert_eq!(
            orchestrator.should_continue_or_spawn(impl_task.as_ref(), "verify the changes"),
            ContinueOrSpawn::Spawn
        );
    }

    #[test]
    fn test_prompt_builder() {
        let directive = PromptBuilder::new("修复 auth bug")
            .with_file_location("src/auth/validate.ts", Some(42), "存在空指针问题")
            .with_action("添加空值检查")
            .with_purpose("修复用户登录问题")
            .with_report_requirement("提交后报告 commit hash")
            .build();

        assert_eq!(directive.description, "修复 auth bug");
        assert!(directive.prompt.contains("src/auth/validate.ts"));
        assert!(directive.prompt.contains("空指针"));
        assert_eq!(directive.purpose, Some("修复用户登录问题".to_string()));
    }
}
