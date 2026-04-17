//! 上下文继承机制
//! 
//! 实现 Fork 子代理的完整对话上下文继承

use crate::message::Message;
use crate::subagent::types::{CacheSafeParams, ToolUseContext, ThinkingConfig};
use std::collections::HashMap;

/// 构建继承的消息历史
/// 
/// Fork 子代理继承父级的完整对话历史
pub fn build_inherited_messages(
    parent_messages: &[Message],
    current_assistant_message: Option<&Message>,
) -> Vec<Message> {
    let mut inherited = Vec::new();
    
    for msg in parent_messages {
        inherited.push(msg.clone());
    }
    
    if let Some(assistant_msg) = current_assistant_message {
        inherited.push(assistant_msg.clone());
    }
    
    inherited
}

/// 构建 User 消息（包含占位符 tool_result 和指令）
pub fn build_user_message_with_placeholder(
    placeholder_result: &str,
    directive: &str,
    tool_use_ids: &[String],
    _fork_config: &crate::subagent::types::ForkSubagentConfig,
) -> Message {
    let mut content = String::new();
    
    for (i, tool_use_id) in tool_use_ids.iter().enumerate() {
        if i > 0 {
            content.push('\n');
        }
        content.push_str(&format!(
            "Tool Result for {}: {}",
            tool_use_id,
            placeholder_result
        ));
    }
    
    content.push_str("\n\n");
    content.push_str(directive);
    
    Message::user_text(content)
}

/// 从父级上下文创建 CacheSafeParams
pub fn create_cache_safe_params(
    system_prompt: String,
    user_context: HashMap<String, String>,
    system_context: HashMap<String, String>,
    tool_use_context: ToolUseContext,
    fork_context_messages: Vec<Message>,
) -> CacheSafeParams {
    CacheSafeParams {
        system_prompt,
        user_context,
        system_context,
        tool_use_context,
        fork_context_messages,
    }
}

/// 克隆内容替换状态
pub fn clone_content_replacement_state(
    parent_state: &crate::subagent::types::ToolUseContext,
) -> ToolUseContext {
    ToolUseContext {
        available_tools: parent_state.available_tools.clone(),
        rendered_system_prompt: parent_state.rendered_system_prompt.clone(),
        thinking_config: parent_state.thinking_config.as_ref().map(|tc| {
            ThinkingConfig {
                enabled: tc.enabled,
                budget_tokens: tc.budget_tokens,
            }
        }),
    }
}

/// 过滤不完整的工具调用
pub fn filter_incomplete_tool_calls(messages: &[Message]) -> Vec<Message> {
    messages.to_vec()
}

/// 获取最后一条 assistant 消息
pub fn get_last_assistant_message(messages: &[Message]) -> Option<&Message> {
    messages.iter().rev().find(|msg| {
        matches!(msg, Message::Assistant(_))
    })
}

/// 提取工具使用块 ID 列表
pub fn extract_tool_use_ids(_message: &Message) -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_inherit_messages() {
        // Placeholder test
    }
}
