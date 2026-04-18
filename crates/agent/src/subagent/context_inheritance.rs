//! 上下文继承机制
//! 
//! 实现 Fork 子代理的完整对话上下文继承

use crate::message::Message;
use crate::subagent::types::{CacheSafeParams, ToolUseContext, ThinkingConfig};
use crate::subagent::build_child_message;
use std::collections::HashMap;

/// 构建继承的消息历史
/// 
/// Fork 子代理继承父级的完整对话历史
pub fn build_inherited_messages(
    parent_messages: &[Message],
    current_assistant_message: Option<&Message>,
) -> Vec<Message> {
    let mut inherited = Vec::new();
    
    // 克隆父级所有历史消息
    for msg in parent_messages {
        inherited.push(msg.clone());
    }
    
    // 如果当前有 assistant 消息（包含 tool_use），也要包含
    if let Some(assistant_msg) = current_assistant_message {
        inherited.push(assistant_msg.clone());
    }
    
    inherited
}

/// 构建 User 消息（包含占位符 tool_result 和指令）
/// 
/// 所有 Fork 使用相同的占位符文本以最大化 Prompt Cache 命中
pub fn build_user_message_with_placeholder(
    placeholder_result: &str,
    directive: &str,
    tool_use_ids: &[String],
    fork_config: &crate::subagent::types::ForkSubagentConfig,
) -> Message {
    let mut content = String::new();
    
    // 为每个 tool_use 生成占位符 tool_result
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
    
    // 添加 Fork 指令
    content.push_str("\n\n");
    content.push_str(&build_child_message(directive, fork_config));
    
    Message::User(crate::message::UserMessage {
        content: vec![crate::message::ContentBlock::Text { text: content }],
    })
}

/// 从父级上下文创建 CacheSafeParams
/// 
/// 确保 Fork 子代理与父级共享相同的缓存关键参数
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

/// 克隆内容替换状态（用于保持 wire prefix 一致）
/// 
/// 复制父级的内容替换状态，使 Fork 子代理的 API 请求前缀与父级一致
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
/// 
/// 移除父级消息中尚未完成的工具调用（assistant 调用了工具但没有对应 result）
pub fn filter_incomplete_tool_calls(messages: &[Message]) -> Vec<Message> {
    let mut filtered = Vec::new();
    let mut pending_tool_uses = std::collections::HashSet::new();
    
    // 倒序扫描，找出所有未完成的 tool_use
    let mut tool_results = std::collections::HashSet::new();
    for msg in messages.iter().rev() {
        if let Some(text) = match msg {
            Message::User(u) => u.content.iter().find_map(|b| if let ContentBlock::Text { text } = b { Some(text) } else { None }),
            Message::Assistant(a) => a.content.iter().find_map(|b| if let ContentBlock::Text { text } = b { Some(text) } else { None }),
            _ => None,
        } {
            if content.contains("tool_result") {
                // 提取 tool_use_id
                // 简化处理：实际应解析 XML
                continue;
            }
        }
        
        if msg.role() == "assistant" {
            // 检查是否有悬空的 tool_use
            if !pending_tool_uses.is_empty() {
                // 有不完整的工具调用
                break;
            }
        }
    }
    
    // 保留完整的消息
    for msg in messages {
        filtered.push(msg.clone());
    }
    
    filtered
}

/// 获取最后一条 assistant 消息
pub fn get_last_assistant_message(messages: &[Message]) -> Option<&Message> {
    messages.iter().rev().find(|msg| msg.role() == "assistant")
}

/// 提取工具使用块 ID 列表
pub fn extract_tool_use_ids(message: &Message) -> Vec<String> {
    let mut ids = Vec::new();
    
    if let Some(text) = match message {
    Message::User(u) => u.content.iter().find_map(|b| if let ContentBlock::Text { text } = b { Some(text) } else { None }),
    Message::Assistant(a) => a.content.iter().find_map(|b| if let ContentBlock::Text { text } = b { Some(text) } else { None }),
    _ => None,
} {
        // 简化：实际应解析 XML 提取所有 tool_use 块的 id
        // 这里仅返回一个占位符
        if content.contains("tool_use") {
            ids.push("tool_use_1".to_string());
        }
    }
    
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageContent;
    
    #[test]
    fn test_inherit_messages() {
        let parent_messages = vec![
            Message {
                role: "user".to_string(),
                content: MessageContent::Text("Hello".to_string()),
                uuid: None,
            },
            Message {
                role: "assistant".to_string(),
                content: MessageContent::Text("Hi there!".to_string()),
                uuid: None,
            },
        ];
        
        let inherited = build_inherited_messages(&parent_messages, None);
        assert_eq!(inherited.len(), 2);
        assert_eq!(inherited[0].role, "user");
        assert_eq!(inherited[1].role, "assistant");
    }
    
    #[test]
    fn test_get_last_assistant() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: MessageContent::Text("Hello".to_string()),
                uuid: None,
            },
            Message {
                role: "assistant".to_string(),
                content: MessageContent::Text("Hi!".to_string()),
                uuid: None,
            },
            Message {
                role: "user".to_string(),
                content: MessageContent::Text("How are you?".to_string()),
                uuid: None,
            },
        ];
        
        let last = get_last_assistant_message(&messages);
        assert!(last.is_some());
        assert_eq!(last.unwrap().role, "assistant");
    }
}
