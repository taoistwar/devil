//! 协调器模式检测
//!
//! 实现协调器模式的启用检测和会话恢复

use crate::coordinator::types::CoordinatorConfig;
use std::env;

/// 检查是否启用协调器模式
///
/// 需要同时满足两个条件：
/// 1. Feature flag 启用
/// 2. 环境变量 CLAUDE_CODE_COORDINATOR_MODE=1
pub fn is_coordinator_mode(config: &CoordinatorConfig) -> bool {
    config.enabled && is_env_coordinator_mode()
}

/// 检查环境变量是否设置
fn is_env_coordinator_mode() -> bool {
    env::var("CLAUDE_CODE_COORDINATOR_MODE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// 会话模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    /// 协调器模式
    Coordinator,
    /// 普通模式
    Normal,
}

impl SessionMode {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "coordinator" => Some(SessionMode::Coordinator),
            "normal" => Some(SessionMode::Normal),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionMode::Coordinator => "coordinator",
            SessionMode::Normal => "normal",
        }
    }
}

/// 匹配会话模式
///
/// 在恢复旧会话时检查存储的模式，如果当前环境变量与存储不一致，
/// 自动翻转环境变量。防止在普通模式下恢复协调器会话（或反之）。
///
/// 返回是否需要切换模式的提示消息
pub fn match_session_mode(stored_mode: Option<SessionMode>) -> Option<String> {
    let stored = match stored_mode {
        Some(mode) => mode,
        None => return None, // 旧会话没有模式记录
    };

    let current_is_coordinator = is_env_coordinator_mode();
    let session_is_coordinator = stored == SessionMode::Coordinator;

    if current_is_coordinator == session_is_coordinator {
        return None; // 模式一致，无需切换
    }

    // 切换环境变量
    if session_is_coordinator {
        env::set_var("CLAUDE_CODE_COORDINATOR_MODE", "1");
    } else {
        env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
    }

    // 返回切换提示
    if session_is_coordinator {
        Some("已进入协调器模式以匹配恢复的会话".to_string())
    } else {
        Some("已退出协调器模式以匹配恢复的会话".to_string())
    }
}

/// 启用协调器模式
pub fn enable_coordinator_mode() {
    env::set_var("CLAUDE_CODE_COORDINATOR_MODE", "1");
}

/// 禁用协调器模式
pub fn disable_coordinator_mode() {
    env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
}

/// 获取协调器用户上下文
///
/// 告知协调者 Worker 可用的工具列表
pub fn get_coordinator_user_context(
    config: &CoordinatorConfig,
) -> std::collections::HashMap<String, String> {
    if !is_coordinator_mode(config) {
        return std::collections::HashMap::new();
    }

    let mut context = std::collections::HashMap::new();
    context.insert(
        "worker_tools_context".to_string(),
        crate::coordinator::types::build_worker_tools_context(config),
    );

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_mode_from_str() {
        assert_eq!(
            SessionMode::from_str("coordinator"),
            Some(SessionMode::Coordinator)
        );
        assert_eq!(
            SessionMode::from_str("Coordinator"),
            Some(SessionMode::Coordinator)
        );
        assert_eq!(SessionMode::from_str("normal"), Some(SessionMode::Normal));
        assert_eq!(SessionMode::from_str("invalid"), None);
    }

    #[test]
    fn test_session_mode_as_str() {
        assert_eq!(SessionMode::Coordinator.as_str(), "coordinator");
        assert_eq!(SessionMode::Normal.as_str(), "normal");
    }

    #[test]
    fn test_is_coordinator_mode() {
        // 两个条件都满足
        let config = CoordinatorConfig {
            enabled: true,
            ..Default::default()
        };
        env::set_var("CLAUDE_CODE_COORDINATOR_MODE", "1");
        assert!(is_coordinator_mode(&config));

        // 只有 feature flag 启用
        env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
        assert!(!is_coordinator_mode(&config));

        // 只有环境变量设置
        env::set_var("CLAUDE_CODE_COORDINATOR_MODE", "1");
        let config_disabled = CoordinatorConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!is_coordinator_mode(&config_disabled));

        // 清理
        env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
    }

    #[test]
    fn test_enable_disable_coordinator_mode() {
        enable_coordinator_mode();
        assert!(is_env_coordinator_mode());

        disable_coordinator_mode();
        assert!(!is_env_coordinator_mode());
    }

    #[test]
    fn test_match_session_mode_no_switch() {
        // 当前普通模式，恢复普通会话
        env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
        let result = match_session_mode(Some(SessionMode::Normal));
        assert!(result.is_none());
        assert!(!is_env_coordinator_mode());

        // 当前协调器模式，恢复协调器会话
        env::set_var("CLAUDE_CODE_COORDINATOR_MODE", "1");
        let result = match_session_mode(Some(SessionMode::Coordinator));
        assert!(result.is_none());
        assert!(is_env_coordinator_mode());

        env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
    }

    #[test]
    fn test_match_session_mode_switch() {
        // 当前普通模式，恢复协调器会话
        env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
        let result = match_session_mode(Some(SessionMode::Coordinator));
        assert!(result.is_some());
        assert!(result.unwrap().contains("已进入协调器模式"));
        assert!(is_env_coordinator_mode());

        // 当前协调器模式，恢复普通会话
        let result = match_session_mode(Some(SessionMode::Normal));
        assert!(result.is_some());
        assert!(result.unwrap().contains("已退出协调器模式"));
        assert!(!is_env_coordinator_mode());
    }

    #[test]
    fn test_match_session_mode_no_stored() {
        // 没有存储的模式
        let result = match_session_mode(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_coordinator_user_context() {
        env::set_var("CLAUDE_CODE_COORDINATOR_MODE", "1");
        let config = CoordinatorConfig {
            enabled: true,
            simple_mode: false,
            scratchpad_dir: None,
            mcp_servers: Vec::new(),
        };

        let context = get_coordinator_user_context(&config);
        assert!(!context.is_empty());
        assert!(context.contains_key("worker_tools_context"));

        // 非协调器模式返回空
        env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
        let config_disabled = CoordinatorConfig {
            enabled: false,
            ..Default::default()
        };
        let empty_context = get_coordinator_user_context(&config_disabled);
        assert!(empty_context.is_empty());
    }
}
