//! Worker Agent 定义
//!
//! 定义协调器模式下的 Worker Agent

use crate::coordinator::types::CoordinatorConfig;
use crate::coordinator::types::{get_worker_tools, WorkerAgent, INTERNAL_ORCHESTRATION_TOOLS};

/// 创建 Worker Agent 定义
pub fn create_worker_agent(config: &CoordinatorConfig) -> WorkerAgent {
    let tools = get_worker_tools(config);

    WorkerAgent {
        agent_type: "worker".to_string(),
        when_to_use: "Worker agent for coordinator mode. Executes research, implementation, and verification tasks autonomously with the full standard tool set.".to_string(),
        tools,
        system_prompt: get_worker_system_prompt().to_string(),
    }
}

/// 获取 Worker 系统提示词
///
/// Worker 的系统提示词告知其职责和行为准则
pub fn get_worker_system_prompt() -> &'static str {
    r#"You are a worker agent spawned by a coordinator. Your job is to complete the task described in the prompt thoroughly and report back with a concise summary of what you did and what you found.

Guidelines:
- Complete the task fully — don't leave it half-done, but don't gold-plate either.
- Use tools proactively: read files, search code, run commands, edit files.
- Be thorough in research: check multiple locations, consider different naming conventions.
- For implementation: make targeted changes, run tests to verify, commit if appropriate.
- Report back with actionable findings — the coordinator will synthesize your results.
- If you encounter errors, investigate and attempt to fix them before reporting failure.
- NEVER create documentation files unless explicitly instructed.

Your response will be delivered to the coordinator as a <task-notification>. Include:
1. A concise summary of what you did
2. Key findings or changes made
3. Any issues encountered and how you handled them
4. File paths and line numbers for important changes
5. Commit hash if you made code changes
"#
}

/// 检查工具是否对 Worker 可用
///
/// 过滤掉内部编排工具（TeamCreate, TeamDelete, SendMessage, SyntheticOutput）
pub fn is_worker_tool_available(tool_name: &str) -> bool {
    !INTERNAL_ORCHESTRATION_TOOLS.contains(&tool_name)
}

/// 获取 Worker 禁用工具列表
pub fn get_forbidden_worker_tools() -> Vec<&'static str> {
    INTERNAL_ORCHESTRATION_TOOLS.to_vec()
}

/// 验证工具访问并返回结果
///
/// 返回 Ok(()) 如果可用，Err(denial_message) 如果被拒绝
pub fn verify_tool_access(tool_name: &str) -> Result<(), String> {
    if is_worker_tool_available(tool_name) {
        Ok(())
    } else {
        Err(format!(
            "工具 '{}' 对 Worker 不可用。Worker 禁止使用内部编排工具: {}",
            tool_name,
            INTERNAL_ORCHESTRATION_TOOLS.join(", ")
        ))
    }
}

/// 检查工具是否为内部编排工具
pub fn is_internal_tool(tool_name: &str) -> bool {
    INTERNAL_ORCHESTRATION_TOOLS.contains(&tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_agent_creation() {
        let config = CoordinatorConfig::default();
        let agent = create_worker_agent(&config);

        assert_eq!(agent.agent_type, "worker");
        assert!(!agent.tools.is_empty());
        assert!(agent.system_prompt.contains("worker agent"));
    }

    #[test]
    fn test_worker_tool_availability() {
        // Worker 可用的工具
        assert!(is_worker_tool_available("Bash"));
        assert!(is_worker_tool_available("Read"));
        assert!(is_worker_tool_available("Edit"));

        // Worker 禁用的工具
        assert!(!is_worker_tool_available("SendMessage"));
        assert!(!is_worker_tool_available("TeamCreate"));
        assert!(!is_worker_tool_available("TeamDelete"));
        assert!(!is_worker_tool_available("SyntheticOutput"));
    }

    #[test]
    fn test_verify_tool_access_allowed() {
        assert!(verify_tool_access("Bash").is_ok());
        assert!(verify_tool_access("Read").is_ok());
        assert!(verify_tool_access("Edit").is_ok());
    }

    #[test]
    fn test_verify_tool_access_denied() {
        let result = verify_tool_access("SendMessage");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不可用"));
    }
}
