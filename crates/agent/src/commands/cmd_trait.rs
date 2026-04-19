//! SlashCommand Trait
//!
//! 定义斜杠命令的基本接口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 命令执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// 是否成功
    pub success: bool,
    /// 输出内容
    pub output: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 附加数据
    pub data: Option<serde_json::Value>,
}

impl CommandResult {
    /// 创建成功结果
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: Some(output.into()),
            error: None,
            data: None,
        }
    }

    /// 创建成功结果带数据
    pub fn success_with_data(output: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            output: Some(output.into()),
            error: None,
            data: Some(data),
        }
    }

    /// 创建失败结果
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(message.into()),
            data: None,
        }
    }
}

/// 命令执行上下文
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// 当前工作目录
    pub cwd: std::path::PathBuf,
    /// 会话 ID
    pub session_id: String,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_default(),
            session_id: String::new(),
        }
    }
}

/// 斜杠命令 trait
/// 所有斜杠命令都实现此 trait
#[async_trait]
pub trait SlashCommand: Send + Sync {
    /// 获取命令名称
    fn name(&self) -> &str;

    /// 获取命令描述
    fn description(&self) -> &str;

    /// 获取命令别名
    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// 获取使用说明
    fn usage(&self) -> &str {
        ""
    }

    /// 执行命令
    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult;
}

/// 宏：简化命令注册
#[macro_export]
macro_rules! register_commands {
    ($registry:expr, $($cmd:ty),*) => {
        $(
            $registry.register(Box::new(<$cmd>::new()))
        )*
    };
}
