//! 钩子注册表
//!
//! 管理钩子的收集、优先级排序和去重

use crate::hooks::events::HookEvent;
use crate::hooks::matcher::HookMatcher;
use crate::hooks::types::HookType;
use std::collections::HashMap;

/// 钩子来源优先级（从高到低）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HookPriority {
    UserSettings = 0,    // 用户设置（最高）
    ProjectSettings = 1, // 项目设置
    LocalSettings = 2,   // 本地设置
    PluginHook = 3,      // 插件钩子
    BuiltinHook = 4,     // 内置钩子
    SessionHook = 5,     // 会话钩子（最低）
}

/// 注册的钩子
#[derive(Debug, Clone)]
pub struct RegisteredHook {
    /// 钩子类型
    pub hook: HookType,
    /// 钩子来源
    pub source: HookSource,
    /// 匹配器
    pub matcher: Option<HookMatcher>,
    /// 优先级
    pub priority: HookPriority,
}

/// 钩子来源
#[derive(Debug, Clone)]
pub struct HookSource {
    /// 来源类型（用户/项目/本地/插件/内置/会话）
    pub source_type: HookSourceType,
    /// 来源路径或标识
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HookSourceType {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    Plugin(String), // 插件根路径
    Builtin,
    Session,
}

/// 钩子注册表
pub struct HookRegistry {
    /// 按事件类型组织的钩子列表
    hooks: HashMap<String, Vec<RegisteredHook>>,
    /// 已注册的回调钩子
    callback_hooks: HashMap<String, Vec<RegisteredHook>>,
}

impl HookRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            callback_hooks: HashMap::new(),
        }
    }

    /// 注册钩子
    pub fn register(&mut self, hook: RegisteredHook) {
        let event_name = "all".to_string(); // 默认适用于所有事件
        self.hooks.entry(event_name).or_default().push(hook);
    }

    /// 注册回调钩子
    pub fn register_callback(&mut self, event_name: impl Into<String>, hook: RegisteredHook) {
        self.callback_hooks
            .entry(event_name.into())
            .or_default()
            .push(hook);
    }

    /// 获取匹配事件的所有钩子（按优先级排序）
    pub fn get_matching_hooks(&self, event: &HookEvent) -> Vec<&RegisteredHook> {
        let mut matching = Vec::new();

        // 获取所有可能匹配的钩子
        for (event_pattern, hooks) in &self.hooks {
            let matcher = HookMatcher::new(event_pattern);
            if matcher.matches(event) {
                for hook in hooks {
                    matching.push(hook);
                }
            }
        }

        // 获取特定事件的回调钩子
        if let Some(hooks) = self.callback_hooks.get(event.name()) {
            for hook in hooks {
                matching.push(hook);
            }
        }

        // 按优先级排序
        matching.sort_by_key(|h| h.priority);

        matching
    }

    /// 清除一次性钩子
    pub fn remove_once_hooks(&mut self) {
        for hooks in self.hooks.values_mut() {
            hooks.retain(|h| {
                if let HookType::Command(cmd) = &h.hook {
                    !cmd.once
                } else if let HookType::Prompt(prompt) = &h.hook {
                    !prompt.once
                } else if let HookType::Agent(agent) = &h.hook {
                    !agent.once
                } else if let HookType::Http(http) = &h.hook {
                    !http.once
                } else {
                    true
                }
            });
        }
    }

    /// 获取所有注册的钩子数量
    pub fn len(&self) -> usize {
        self.hooks.values().map(|v| v.len()).sum()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.hooks.values().all(|v| v.is_empty())
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 钩子配置快照（用于在设置变更后保留原始配置）
#[derive(Debug, Clone)]
pub struct HookConfigSnapshot {
    /// 用户设置的钩子
    pub user_hooks: Vec<RegisteredHook>,
    /// 项目设置的钩子
    pub project_hooks: Vec<RegisteredHook>,
    /// 本地设置的钩子
    pub local_hooks: Vec<RegisteredHook>,
}

impl HookConfigSnapshot {
    /// 创建空快照
    pub fn empty() -> Self {
        Self {
            user_hooks: Vec::new(),
            project_hooks: Vec::new(),
            local_hooks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::types::{CommandHook, ShellType};

    #[test]
    fn test_registry_create() {
        let registry = HookRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_hook() {
        let mut registry = HookRegistry::new();
        let hook = RegisteredHook {
            hook: HookType::Command(CommandHook {
                command: "echo test".to_string(),
                shell: ShellType::Bash,
                condition: None,
                timeout: None,
                status_message: None,
                once: false,
                r#async: false,
                async_rewake: false,
            }),
            source: HookSource {
                source_type: HookSourceType::UserSettings,
                path: None,
            },
            matcher: None,
            priority: HookPriority::UserSettings,
        };

        registry.register(hook);
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_priority_ordering() {
        let mut registry = HookRegistry::new();

        // 注册不同优先级的钩子
        let sources = vec![
            HookPriority::BuiltinHook,
            HookPriority::UserSettings,
            HookPriority::PluginHook,
        ];

        for (i, priority) in sources.into_iter().enumerate() {
            let hook = RegisteredHook {
                hook: HookType::Command(CommandHook {
                    command: format!("echo {}", i),
                    shell: ShellType::Bash,
                    condition: None,
                    timeout: None,
                    status_message: None,
                    once: false,
                    r#async: false,
                    async_rewake: false,
                }),
                source: HookSource {
                    source_type: HookSourceType::Builtin,
                    path: None,
                },
                matcher: None,
                priority: priority,
            };
            registry.register(hook);
        }

        // 获取钩子并验证优先级顺序
        let event = HookEvent::UserPromptSubmit {
            message: "test".to_string(),
        };
        let hooks = registry.get_matching_hooks(&event);

        assert_eq!(hooks.len(), 3);
        // 验证按优先级排序（UserSettings 最高）
        assert_eq!(hooks[0].priority, HookPriority::UserSettings);
        assert_eq!(hooks[1].priority, HookPriority::PluginHook);
        assert_eq!(hooks[2].priority, HookPriority::BuiltinHook);
    }

    #[test]
    fn test_remove_once_hooks() {
        let mut registry = HookRegistry::new();

        // 注册一次性钩子
        let once_hook = RegisteredHook {
            hook: HookType::Command(CommandHook {
                command: "echo once".to_string(),
                shell: ShellType::Bash,
                condition: None,
                timeout: None,
                status_message: None,
                once: true,
                r#async: false,
                async_rewake: false,
            }),
            source: HookSource {
                source_type: HookSourceType::UserSettings,
                path: None,
            },
            matcher: None,
            priority: HookPriority::UserSettings,
        };

        // 注册非一次性钩子
        let persist_hook = RegisteredHook {
            hook: HookType::Command(CommandHook {
                command: "echo persist".to_string(),
                shell: ShellType::Bash,
                condition: None,
                timeout: None,
                status_message: None,
                once: false,
                r#async: false,
                async_rewake: false,
            }),
            source: HookSource {
                source_type: HookSourceType::UserSettings,
                path: None,
            },
            matcher: None,
            priority: HookPriority::UserSettings,
        };

        registry.register(once_hook);
        registry.register(persist_hook);
        assert_eq!(registry.len(), 2);

        registry.remove_once_hooks();
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_callback_hooks() {
        let mut registry = HookRegistry::new();

        let hook = RegisteredHook {
            hook: HookType::Command(CommandHook {
                command: "echo callback".to_string(),
                shell: ShellType::Bash,
                condition: None,
                timeout: None,
                status_message: None,
                once: false,
                r#async: false,
                async_rewake: false,
            }),
            source: HookSource {
                source_type: HookSourceType::Builtin,
                path: None,
            },
            matcher: None,
            priority: HookPriority::BuiltinHook,
        };

        registry.register_callback("pre_tool_use", hook);

        let event = HookEvent::PreToolUse {
            tool_name: "Bash".to_string(),
            tool_input: std::collections::HashMap::new(),
        };

        let hooks = registry.get_matching_hooks(&event);
        assert!(!hooks.is_empty());
    }
}
