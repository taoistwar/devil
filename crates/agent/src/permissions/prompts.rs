//! 权限提示模块
//!
//! 实现交互式权限确认机制：
//! - 使用 tokio channel 异步通知
//! - 支持多决策者竞争（hook、classifier、user）
//! - ResolveOnce 模式保证原子化决策

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// 权限提示消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPrompt {
    /// 工具名称
    pub tool_name: String,
    /// 原始输入摘要
    pub input_summary: String,
    /// 提示内容
    pub prompt: String,
    /// 决策来源
    pub source: DecisionSource,
    /// 创建时间戳
    pub created_at: u64,
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

/// 权限提示响应
#[derive(Debug, Clone)]
pub struct PermissionResponse {
    /// 是否允许
    pub allowed: bool,
    /// 是否永久允许（仅 user 来源有效）
    pub permanent: bool,
    /// 拒绝原因（如果拒绝）
    pub reason: Option<String>,
}

/// 权限提示器
#[derive(Clone)]
pub struct PermissionPrompter {
    /// 广播频道用于向所有观察者发送提示
    prompt_tx: broadcast::Sender<PermissionPrompt>,
    /// 响应发送端
    response_tx: Arc<RwLock<Option<mpsc::Sender<PermissionResponse>>>>,
}

impl PermissionPrompter {
    /// 创建新的权限提示器
    pub fn new() -> Self {
        let (prompt_tx, _) = broadcast::channel(100);
        Self {
            prompt_tx,
            response_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// 创建带缓冲大小的权限提示器
    pub fn with_capacity(capacity: usize) -> Self {
        let (prompt_tx, _) = broadcast::channel(capacity);
        Self {
            prompt_tx,
            response_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// 发送权限提示
    ///
    /// 返回一个 channel receiver 用于接收响应
    pub async fn send_prompt(
        &self,
        prompt: PermissionPrompt,
    ) -> mpsc::Receiver<PermissionResponse> {
        let (response_tx, response_rx) = mpsc::channel(1);

        // 存储发送端，以便后续可以使用
        let mut guard = self.response_tx.write().await;
        *guard = Some(response_tx.clone());

        // 广播提示给所有观察者
        let _ = self.prompt_tx.send(prompt);

        response_rx
    }

    /// 尝试发送快速响应（用于 hook 或 classifier）
    ///
    /// 如果已经有其他决策者先一步发送响应，返回 false
    pub async fn try_send_response(&self, response: PermissionResponse) -> bool {
        let guard = self.response_tx.read().await;
        if let Some(ref tx) = *guard {
            tx.send(response).await.is_ok()
        } else {
            false
        }
    }

    /// 订阅提示频道
    pub fn subscribe(&self) -> broadcast::Receiver<PermissionPrompt> {
        self.prompt_tx.subscribe()
    }
}

impl Default for PermissionPrompter {
    fn default() -> Self {
        Self::new()
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
    pub fn set(&self, value: T) -> Result<(), &'static str> {
        if self.claimed.load(std::sync::atomic::Ordering::SeqCst) {
            let mut inner = self.value.lock().unwrap();
            *inner = Some(value);
            Ok(())
        } else {
            Err("ResolveOnce already claimed by another caller")
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

/// 权限确认竞决策
///
/// 用于管理多个决策者（hook、classifier、user）之间的竞争
pub struct PermissionDecisionRace {
    /// 解决状态
    resolve_once: ResolveOnce<PermissionResponse>,
}

impl PermissionDecisionRace {
    /// 创建新的竞决策
    pub fn new() -> Self {
        Self {
            resolve_once: ResolveOnce::new(None),
        }
    }

    /// 尝试由 hook 做出决策
    pub fn try_hook_decide(&self, allowed: bool, reason: Option<String>) -> bool {
        if self.resolve_once.claim() {
            let response = PermissionResponse {
                allowed,
                permanent: false,
                reason,
            };
            let _ = self.resolve_once.set(response);
            true
        } else {
            false
        }
    }

    /// 尝试由 classifier 做出决策
    pub fn try_classifier_decide(&self, allowed: bool) -> bool {
        if self.resolve_once.claim() {
            let response = PermissionResponse {
                allowed,
                permanent: false,
                reason: None,
            };
            let _ = self.resolve_once.set(response);
            true
        } else {
            false
        }
    }

    /// 尝试由 user 做出决策
    pub fn try_user_decide(&self, allowed: bool, permanent: bool, reason: Option<String>) -> bool {
        if self.resolve_once.claim() {
            let response = PermissionResponse {
                allowed,
                permanent,
                reason,
            };
            let _ = self.resolve_once.set(response);
            true
        } else {
            false
        }
    }

    /// 获取最终决策（如果已解决）
    pub fn get_decision(&self) -> Option<PermissionResponse> {
        self.resolve_once.get()
    }

    /// 判断是否已解决
    pub fn is_resolved(&self) -> bool {
        self.resolve_once.is_resolved()
    }

    /// 等待决策超时时间
    pub async fn wait_for_decision(
        &self,
        timeout: std::time::Duration,
    ) -> Option<PermissionResponse> {
        let start = std::time::Instant::now();
        while !self.is_resolved() {
            if start.elapsed() > timeout {
                return None;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        self.get_decision()
    }
}

impl Default for PermissionDecisionRace {
    fn default() -> Self {
        Self::new()
    }
}

/// 权限提示管理器
///
/// 管理全局的权限提示和决策
pub struct PermissionPromptManager {
    /// 当前活跃的 prompter
    prompter: RwLock<Option<PermissionPrompter>>,
}

impl PermissionPromptManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            prompter: RwLock::new(None),
        }
    }

    /// 设置活跃的 prompter
    pub async fn set_prompter(&self, prompter: PermissionPrompter) {
        let mut guard = self.prompter.write().await;
        *guard = Some(prompter);
    }

    /// 获取当前的 prompter
    pub async fn get_prompter(&self) -> Option<PermissionPrompter> {
        let guard = self.prompter.read().await;
        guard.clone()
    }

    /// 清除当前的 prompter
    pub async fn clear_prompter(&self) {
        let mut guard = self.prompter.write().await;
        *guard = None;
    }
}

impl Default for PermissionPromptManager {
    fn default() -> Self {
        Self::new()
    }
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
    fn test_permission_decision_race() {
        let race = PermissionDecisionRace::new();

        // 初始状态未解决
        assert!(!race.is_resolved());

        // hook 可以先决策
        assert!(race.try_hook_decide(true, None));
        assert!(race.is_resolved());
        assert!(race.get_decision().unwrap().allowed);

        // 后续决策者无法覆盖
        assert!(!race.try_user_decide(false, false, None));
        assert!(race.get_decision().unwrap().allowed);
    }

    #[tokio::test]
    async fn test_permission_prompter() {
        let prompter = PermissionPrompter::new();

        let prompt = PermissionPrompt {
            tool_name: "Bash".to_string(),
            input_summary: "rm -rf /tmp/test".to_string(),
            prompt: "Destructive command detected".to_string(),
            source: DecisionSource::Classifier,
            created_at: 0,
        };

        let rx = prompter.send_prompt(prompt.clone()).await;

        // 检查是否能接收到广播
        let mut subscribe_rx = prompter.subscribe();
        let received = subscribe_rx.recv().await.unwrap();
        assert_eq!(received.tool_name, "Bash");
    }
}
