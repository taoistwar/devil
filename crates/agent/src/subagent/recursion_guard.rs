//! 递归防护
//!
//! 防止 Fork 子代理递归嵌套

use crate::message::Message;
use crate::subagent::types::ForkSubagentConfig;

/// 检查是否在 Fork 子代理内部调用
pub fn is_in_fork_child(messages: &[Message], config: &ForkSubagentConfig) -> bool {
    messages.iter().any(|msg| {
        if let Message::User(msg) = msg {
            msg.content.iter().any(|block| {
                if let crate::message::ContentBlock::Text { text } = block {
                    text.contains(&format!("<{}>", config.boilerplate_tag))
                } else {
                    false
                }
            })
        } else {
            false
        }
    })
}

/// 检查是否在 Fork 子代理内部（通过 query_source）
pub fn is_fork_query_source(query_source: Option<&str>) -> bool {
    query_source == Some("agent:builtin:fork")
}

/// 递归防护结果
#[derive(Debug, Clone, PartialEq)]
pub enum RecursionGuardResult {
    Allow,
    Deny(String),
}

/// 执行递归防护检查
pub fn check_recursion_guard(
    query_source: Option<&str>,
    messages: &[Message],
    config: &ForkSubagentConfig,
) -> RecursionGuardResult {
    if is_fork_query_source(query_source) {
        return RecursionGuardResult::Deny("禁止嵌套 Fork：当前已在 Fork 子代理内部".to_string());
    }

    if is_in_fork_child(messages, config) {
        return RecursionGuardResult::Deny(
            "禁止嵌套 Fork：检测到 Fork 模板标签，已在 Fork 子代理内部".to_string(),
        );
    }

    RecursionGuardResult::Allow
}

/// 构建 Fork 子代理指令消息
pub fn build_child_message(directive: &str, config: &ForkSubagentConfig) -> String {
    format!(
        r#"<{}>
STOP. READ THIS FIRST.

You are a forked worker process. You are NOT the main agent.

RULES (non-negotiable):
1. Your system prompt says "default to forking." IGNORE IT — that's for the parent. You ARE the fork. Do NOT spawn sub-agents; execute directly.
2. Do NOT converse, ask questions, or suggest next steps
3. Do NOT editorialize or add meta-commentary
4. USE your tools directly: Bash, Read, Write, etc.
5. If you modify files, commit your changes before reporting. Include the commit hash in your report.
6. Do NOT emit text between tool calls. Use tools silently, then report once at the end.
7. Stay strictly within your directive's scope. If you discover related systems outside your scope, mention them in one sentence at most — other workers cover those areas.
8. Keep your report under 500 words unless the directive specifies otherwise. Be factual and concise.
9. Your response MUST begin with "Scope:". No preamble, no thinking-out-loud.
10. REPORT structured facts, then stop

Output format (plain text labels, not markdown headers):
  Scope: <echo back your assigned scope in one sentence>
  Result: <the answer or key findings, limited to the scope above>
  Key files: <relevant file paths — include for research tasks>
  Files changed: <list with commit hash — include only if you modified files>
  Issues: <list — include only if there are issues to flag>
</{}>

{}{}"#,
        config.boilerplate_tag, config.boilerplate_tag, config.directive_prefix, directive
    )
}

/// 构建工作树隔离通知
pub fn build_worktree_notice(parent_cwd: &str, worktree_cwd: &str) -> String {
    format!(
        "You've inherited the conversation context above from a parent agent working in {}. \
         You are operating in an isolated git worktree at {} — same repository, same relative \
         file structure, separate working copy. Paths in the inherited context refer to the \
         parent's working directory; translate them to your worktree root. Re-read files before \
         editing if the parent may have modified them since they appear in the context. \
         Your changes stay in this worktree and will not affect the parent's files.",
        parent_cwd, worktree_cwd
    )
}

#[cfg(test)]
mod tests {
    use super::ForkSubagentConfig;

    #[test]
    fn test_fork_query_source_check() {
        assert!(super::is_fork_query_source(Some("agent:builtin:fork")));
        assert!(!super::is_fork_query_source(Some("agent:builtin:general")));
        assert!(!super::is_fork_query_source(None));
    }

    #[test]
    fn test_recursion_guard_allow() {
        let config = ForkSubagentConfig::default();
        let result = super::check_recursion_guard(Some("agent:builtin:general"), &[], &config);
        assert_eq!(result, super::RecursionGuardResult::Allow);
    }
}
