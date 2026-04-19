//! /help 命令 - 显示帮助信息

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /help 命令
pub struct HelpCommand;

impl HelpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelpCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "显示帮助信息"
    }

    fn aliases(&self) -> &[&str] {
        &["?"]
    }

    fn usage(&self) -> &str {
        "/help [command]"
    }

    async fn execute(&self, _ctx: &CommandContext, args: &[&str]) -> CommandResult {
        if let Some(cmd) = args.first() {
            self.show_command_help(cmd)
        } else {
            self.show_all_help()
        }
    }
}

impl HelpCommand {
    fn show_all_help(&self) -> CommandResult {
        let help_text = r#"
Devil Agent - AI 编程助手

用法: /command [args]

核心命令:
  /help [command]     显示帮助信息
  /compact            手动压缩上下文
  /model [model]      切换 AI 模型
  /clear              清除当前对话
  /exit               退出程序
  /resume             恢复之前的会话
  /doctor             运行系统诊断
  /cost               查看 API 使用费用

配置命令:
  /config             打开配置管理
  /login              登录认证
  /logout             登出认证
  /theme [theme]      切换主题

高级命令:
  /mcp                MCP 服务器管理
  /hooks              Hook 管理
  /skills             技能管理
  /tasks              任务管理
  /memory             记忆管理
  /permissions        权限管理

编辑命令:
  /diff [file]        查看文件差异
  /review             代码审查
  /plan               进入计划模式
  /vim                Vim 编辑模式

协作命令:
  /share              分享当前会话
  /voice              语音输入模式
  /stickers           贴纸

输入 /help <command> 查看特定命令的帮助
"#;
        CommandResult::success(help_text.trim())
    }

    fn show_command_help(&self, command: &str) -> CommandResult {
        let help_text = match command {
            "help" => {
                r#"/help [command]
显示帮助信息
  - 无参数: 显示所有可用命令
  - 有参数: 显示指定命令的帮助

示例:
  /help
  /help compact"#
            }
            "compact" => {
                r#"/compact
手动压缩上下文，释放 token 空间
这会在需要时自动触发，但可以手动强制执行"#
            }
            "model" => {
                r#"/model [model-name]
切换 AI 模型

示例:
  /model claude-sonnet-4
  /model claude-opus"#
            }
            "clear" => {
                r#"/clear
清除当前对话历史
警告: 此操作不可逆"#
            }
            _ => return CommandResult::error(format!("未知命令: {}", command)),
        };
        CommandResult::success(help_text)
    }
}
