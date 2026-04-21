//! 钩子匹配器
//!
//! 实现钩子事件的匹配逻辑

use crate::hooks::events::HookEvent;
use crate::hooks::types::CommandHook;

/// 钩子匹配器
#[derive(Debug, Clone)]
pub struct HookMatcher {
    /// 匹配模式（精确、管道分隔、正则）
    pattern: String,
    /// 编译后的正则（如果有）
    regex: Option<regex::Regex>,
}

impl HookMatcher {
    /// 创建新的匹配器
    pub fn new(pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();

        // 检测是否为正则表达式
        let regex = if pattern.starts_with('^') || pattern.contains(".*") || pattern.contains("\\")
        {
            regex::Regex::new(&pattern).ok()
        } else {
            None
        };

        Self { pattern, regex }
    }

    /// 检查事件是否匹配
    pub fn matches(&self, event: &HookEvent) -> bool {
        // 空模式或通配符匹配所有
        if self.pattern.is_empty() || self.pattern == "*" {
            return true;
        }

        // 获取事件的匹配字段
        let Some(match_field) = event.match_field() else {
            return false;
        };

        // 使用正则匹配
        if let Some(ref regex) = self.regex {
            return regex.is_match(&match_field);
        }

        // 管道分隔的多值匹配
        if self.pattern.contains('|') {
            return self.pattern.split('|').any(|p| p.trim() == match_field);
        }

        // 精确匹配
        self.pattern == match_field
    }

    /// 检查 if 条件是否满足
    pub fn matches_condition(
        &self,
        hook: &CommandHook,
        event: &HookEvent,
        tool_input: Option<&serde_json::Value>,
    ) -> bool {
        // 没有条件则默认匹配
        let Some(condition) = &hook.condition else {
            return true;
        };

        // 解析条件中的工具名和参数模式
        // 例如："Bash(git *)" 或 "Write(*.ts)"
        if let Some((tool_name, arg_pattern)) = Self::parse_condition(condition) {
            // 检查工具名是否匹配
            if let Some(event_tool_name) = event.match_field() {
                if event_tool_name != tool_name {
                    return false;
                }

                // 检查参数是否匹配
                if let Some(pattern) = arg_pattern {
                    if let Some(input) = tool_input {
                        return Self::match_arguments(input, &pattern);
                    }
                }

                return true;
            }
        }

        false
    }

    /// 解析条件字符串
    /// 返回 (tool_name, arg_pattern)
    fn parse_condition(condition: &str) -> Option<(String, Option<String>)> {
        // 格式：ToolName(pattern)
        let start = condition.find('(')?;
        let end = condition.rfind(')')?;

        let tool_name = condition[..start].to_string();
        let inner = if end > start + 1 {
            Some(condition[start + 1..end].to_string())
        } else {
            None
        };

        Some((tool_name, inner))
    }

    /// 匹配参数模式
    fn match_arguments(input: &serde_json::Value, pattern: &str) -> bool {
        // 简单通配符匹配
        if pattern == "*" {
            return true;
        }

        // 检查是否以特定前缀开头
        if let Some(prefix) = pattern.strip_suffix('*') {
            if let Some(input_str) = input.as_str() {
                return input_str.starts_with(prefix);
            }
        }

        // 精确匹配
        if let Some(input_str) = input.as_str() {
            return input_str == pattern;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pattern() {
        let matcher = HookMatcher::new("");
        let event = HookEvent::UserPromptSubmit {
            message: "test".to_string(),
        };
        assert!(matcher.matches(&event));
    }

    #[test]
    fn test_wildcard() {
        let matcher = HookMatcher::new("*");
        let event = HookEvent::PreToolUse {
            tool_name: "Bash".to_string(),
            tool_input: std::collections::HashMap::new(),
        };
        assert!(matcher.matches(&event));
    }

    #[test]
    fn test_exact_match() {
        let matcher = HookMatcher::new("Bash");
        let event = HookEvent::PreToolUse {
            tool_name: "Bash".to_string(),
            tool_input: std::collections::HashMap::new(),
        };
        assert!(matcher.matches(&event));

        let event2 = HookEvent::PreToolUse {
            tool_name: "Read".to_string(),
            tool_input: std::collections::HashMap::new(),
        };
        assert!(!matcher.matches(&event2));
    }

    #[test]
    fn test_pipe_delimited() {
        let matcher = HookMatcher::new("Bash|Read|Write");
        let event = HookEvent::PreToolUse {
            tool_name: "Read".to_string(),
            tool_input: std::collections::HashMap::new(),
        };
        assert!(matcher.matches(&event));
    }
}
