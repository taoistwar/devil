//! 内置钩子
//! 
//! 提供系统预设的钩子实现

use crate::hooks::{HookType, HookRegistry, HookPriority, HookSource, HookSourceType};

/// 注册内置钩子
pub fn register_builtin_hooks(registry: &mut HookRegistry) {
    // PreToolUse: Git 提交前检查
    registry.register(crate::hooks::RegisteredHook {
        hook: HookType::Command(crate::hooks::CommandHook {
            command: "echo '检查 Git 提交...'".to_string(),
            shell: crate::hooks::ShellType::Bash,
            condition: Some("Bash(git commit*)".to_string()),
            timeout: Some(30),
            status_message: Some("运行提交前检查...".to_string()),
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
    });
    
    // SessionStart: 加载项目说明
    registry.register(crate::hooks::RegisteredHook {
        hook: HookType::Prompt(crate::hooks::PromptHook {
            prompt: "请提醒用户当前项目的工作目录和主要技术栈".to_string(),
            condition: None,
            timeout: Some(10),
            model: Some("claude-haiku".to_string()),
            status_message: Some("加载项目说明...".to_string()),
            once: true,
        }),
        source: HookSource {
            source_type: HookSourceType::Builtin,
            path: None,
        },
        matcher: None,
        priority: HookPriority::BuiltinHook,
    });
}

/// 获取内置钩子数量
pub fn builtin_hook_count() -> usize {
    2
}
