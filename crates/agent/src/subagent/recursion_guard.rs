//! 递归防护
//! 
//! 防止 Fork 子代理递归嵌套

use crate::message::Message;
use crate::subagent::types::ForkSubagentConfig;

/// 检查是否在 Fork 子代理内部调用
/// 
/// 扫描消息历史中是否包含 Fork 模板标签
pub fn is_in_fork_child(messages: &[Message], config: &ForkSubagentConfig) -> bool {
    messages.iter().any(|msg| {
        if msg.role != "user" {
            return false;
        }
        
        let content = &msg.content;
        if let Some(text) = content.as_text() {
            return text.contains(&format!("<{}>", config.boilerplate_tag));
        }
        
        false
    })
}

/// 检查是否在 Fork 子代理内部（通过 query_source）
/// 
/// query_source 标记为 'agent:builtin:fork' 时拒绝递归
pub fn is_fork_query_source(query_source: Option<&str>) -> bool {
    query_source == Some("agent:builtin:fork")
}

/// 递归防护结果
#[derive(Debug, Clone, PartialEq)]
pub enum RecursionGuardResult {
    /// 允许执行
    Allow,
    /// 拒绝执行（原因）
    Deny(String),
}

/// 执行递归防护检查
/// 
/// 两层防护：
/// 1. query_source 检查（抗自动压缩）
/// 2. 消息扫描（检测 Fork 模板标签）
pub fn check_recursion_guard(
    query_source: Option<&str>,
    messages: &[Message],
    config: &ForkSubagentConfig,
) -> RecursionGuardResult {
    // 第一层：query_source 检查
    if is_fork_query_source(query_source) {
        return RecursionGuardResult::Deny(
            "禁止嵌套 Fork：当前已在 Fork 子代理内部".to_string()
        );
    }
    
    // 第二层：消息扫描
    if is_in_fork_child(messages, config) {
        return RecursionGuardResult::Deny(
            "禁止嵌套 Fork：检测到 Fork 模板标签，已在 Fork 子代理内部".to_string()
        );
    }
    
    RecursionGuardResult::Allow
}

/// 构建 Fork 子代理指令消息
/// 
/// 生成包含 `<fork-boilerplate>` 标签的指令，告知子代理其角色和规则
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
        config.boilerplate_tag,
        config.boilerplate_tag,
        config.directive_prefix,
        directive
    )
}

/// 构建工作树隔离通知
/// 
/// 当 Fork + Worktree 组合时，告知子代理路径转换规则
pub fn build_worktree_notice(parent_cwd: &str, worktree_cwd: &str) -> String {
    format!(
        "You've inherited the conversation context above from a parent agent working in {}. \
         You are operating in an isolated git worktree at {} — same repository, same relative \
         file structure, separate working copy. Paths in the inherited context refer to the \
         parent's working directory; translate them to your worktree root. Re-read files before \
         editing if the parent may have modified them since they appear in the context. \
         Your changes stay in this worktree and will not affect the parent's files.",
        parent_cwd,
        worktree_cwd
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Message, MessageContent};
    
    #[test]
    fn test_fork_query_source_check() {
        assert!(is_fork_query_source(Some("agent:builtin:fork")));
        assert!(!is_fork_query_source(Some("agent:builtin:general")));
        assert!(!is_fork_query_source(None));
    }
    
    #[test]
    fn test_is_in_fork_child() {
        let config = ForkSubagentConfig::default();
        let fork_message = Message {
            role: "user".to_string(),
            content: MessageContent::Text(format!(
                "<{}>Some fork directive</{}>",
                config.boilerplate_tag,
                config.boilerplate_tag
            )),
            uuid: None,
        };
        
        let normal_message = Message {
            role: "user".to_string(),
            content: MessageContent::Text("Normal user message".to_string()),
            uuid: None,
        };
        
        assert!(is_in_fork_child(&[fork_message.clone()], &config));
        assert!(!is_in_fork_child(&[normal_message.clone()], &config));
    }
    
    #[test]
    fn test_recursion_guard_check() {
        let config = ForkSubagentConfig::default();
        
        // 正常情况应该允许
        let result = check_recursion_guard(
            Some("agent:builtin:general"),
            &[],
            &config,
        );
        assert_eq!(result, RecursionGuardResult::Allow);
        
        // query_source 为 fork 应该拒绝
        let result = check_recursion_guard(
            Some("agent:builtin:fork"),
            &[],
            &config,
        );
        assert_eq!(result, RecursionGuardResult::Deny(_));
    }
    
    #[test]
    fn test_recursion_guard_message_scan() {
        let config = ForkSubagentConfig::default();
        let fork_message = Message {
            role: "user".to_string(),
            content: MessageContent::Text(format!(
                "<{}>Directive</{}>",
                config.boilerplate_tag,
                config.boilerplate_tag
            )),
            uuid: None,
        };
        
        let result = check_recursion_guard(
            None,
            &[fork_message],
            &config,
        );
        assert_eq!(result, RecursionGuardResult::Deny(_));
    }
    
    #[test]
    fn test_build_child_message_format() {
        let config = ForkSubagentConfig::default();
        let directive = "Analyze the codebase";
        let child_message = build_child_message(directive, &config);
        
        // 应该包含模板标签
        assert!(child_message.contains(&format!("<{}>", config.boilerplate_tag)));
        assert!(child_message.contains("STOP. READ THIS FIRST."));
        assert!(child_message.contains("Scope:"));
        assert!(child_message.contains(directive));
    }
    
    #[test]
    fn test_build_worktree_notice() {
        let notice = build_worktree_notice(
            "/workspace/parent",
            "/workspace/worktree"
        );
        
        assert!(notice.contains("/workspace/parent"));
        assert!(notice.contains("/workspace/worktree"));
        assert!(notice.contains("git worktree"));
        assert!(notice.contains("isolated"));
    }
    
    #[test]
    fn test_two_layer_protection() {
        let config = ForkSubagentConfig::default();
        
        // 第一层防护：query_source
        let result1 = check_recursion_guard(
            Some("agent:builtin:fork"),
            &[],
            &config,
        );
        assert!(matches!(result1, RecursionGuardResult::Deny(_)));
        
        // 第二层防护：消息扫描
        let fork_msg = Message {
            role: "user".to_string(),
            content: MessageContent::Text(format!(
                "<{}>test</{}>",
                config.boilerplate_tag,
                config.boilerplate_tag
            )),
            uuid: None,
        };
        let result2 = check_recursion_guard(
            None,
            &[fork_msg],
            &config,
        );
        assert!(matches!(result2, RecursionGuardResult::Deny(_)));
        
        // 两层都通过才允许
        let result3 = check_recursion_guard(
            None,
            &[],
            &config,
        );
        assert_eq!(result3, RecursionGuardResult::Allow);
    }
}
