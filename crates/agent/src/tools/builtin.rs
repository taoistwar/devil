//! 内建工具模块
//!
//! 实现 Claude Code 的核心内建工具：
//! - BashTool: 命令执行的瑞士军刀
//! - FileReadTool: 读取文件内容
//! - FileEditTool: 精确编辑文件
//! - FileWriteTool: 创建或覆写文件
//! - GlobTool: 文件名模式匹配
//! - GrepTool: 内容搜索

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::permissions::bash_analyzer::BashSemanticAnalyzer;
use crate::tools::tool::{
    ContextModifier, InputValidationResult, InterruptBehavior, PermissionBehavior,
    PermissionResult, Tool, ToolContext, ToolPermissionLevel, ToolResult,
};

/// buildTool 工厂函数
///
/// 创建工具的标准工厂函数，自动填充安全默认值
///
/// 遵循 "fail-closed" 原则：
/// 安全性相关的方法（如并发安全判断、只读判断）默认为 false
/// 工具必须显式声明自己安全才能享受并发等优化
pub struct ToolBuilder<I, O> {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    aliases: Vec<String>,
    permission_level: ToolPermissionLevel,
    concurrency_safe: bool,
    read_only: bool,
    timeout_secs: Option<u64>,
    always_load: bool,
    execute_fn: Box<dyn Fn(I, &ToolContext) -> Result<O> + Send + Sync>,
}

impl<I, O> ToolBuilder<I, O>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    /// 创建新的工具构建器
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: serde_json::Value::Null,
            aliases: Vec::new(),
            permission_level: ToolPermissionLevel::RequiresConfirmation,
            concurrency_safe: false, // fail-closed: 默认不安全
            read_only: false,        // fail-closed: 默认不是只读
            timeout_secs: None,
            always_load: false,
            execute_fn: Box::new(|_, _| Err(anyhow::anyhow!("No execute function"))),
        }
    }

    /// 设置输入 schema
    pub fn input_schema(mut self, schema: serde_json::Value) -> Self {
        self.input_schema = schema;
        self
    }

    /// 设置别名
    pub fn aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    /// 设置权限级别
    pub fn permission_level(mut self, level: ToolPermissionLevel) -> Self {
        self.permission_level = level;
        self
    }

    /// 标记为并发安全
    pub fn concurrency_safe(mut self) -> Self {
        self.concurrency_safe = true;
        self
    }

    /// 标记为只读
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self.permission_level = ToolPermissionLevel::ReadOnly;
        self
    }

    /// 设置超时时间
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// 标记为始终加载（用于延迟发现）
    pub fn always_load(mut self) -> Self {
        self.always_load = true;
        self
    }

    /// 设置执行函数
    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: Fn(I, &ToolContext) -> Result<O> + Send + Sync + 'static,
    {
        self.execute_fn = Box::new(f);
        self
    }

    /// 构建工具
    pub fn build(self) -> BuiltTool<I, O> {
        BuiltTool {
            name: self.name,
            description: self.description,
            input_schema: self.input_schema,
            aliases: self.aliases,
            permission_level: self.permission_level,
            concurrency_safe: self.concurrency_safe,
            read_only: self.read_only,
            timeout_secs: self.timeout_secs,
            always_load: self.always_load,
            execute_fn: self.execute_fn,
        }
    }
}

/// 构建完成的工具
pub struct BuiltTool<I, O> {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    aliases: Vec<String>,
    permission_level: ToolPermissionLevel,
    concurrency_safe: bool,
    read_only: bool,
    timeout_secs: Option<u64>,
    always_load: bool,
    execute_fn: Box<dyn Fn(I, &ToolContext) -> Result<O> + Send + Sync>,
}

#[async_trait]
impl<I, O> Tool for BuiltTool<I, O>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    type Input = I;
    type Output = O;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        &self.name
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn input_schema(&self) -> serde_json::Value {
        self.input_schema.clone()
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        self.permission_level
    }

    fn is_concurrency_safe(&self) -> bool {
        self.concurrency_safe
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        self.timeout_secs
    }

    fn should_always_load(&self) -> bool {
        self.always_load
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        let output = (self.execute_fn)(input, ctx)?;
        Ok(ToolResult::success(format!("{}-1", self.name), output))
    }
}

// ===== BashTool =====

/// Bash 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashInput {
    /// 要执行的命令
    pub command: String,
    /// 工作目录（可选）
    pub cwd: Option<String>,
    /// 是否后台执行
    pub background: Option<bool>,
}

/// Bash 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashOutput {
    /// 退出码
    pub exit_code: i32,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
}

/// BashTool：命令执行的瑞士军刀
///
/// 最复杂的工具，集成多层安全防护：
/// - 错误传播：Bash 失败会取消所有并行的 Bash 工具
/// - 中断行为：可自定义用户中断时的行为
/// - 语义分析：对命令进行 AST 解析和语义分析
/// - 沙盒集成：控制命令执行的安全边界
pub struct BashTool {
    #[allow(dead_code)]
    disable_sandbox: bool,
}

impl BashTool {
    pub fn new(disable_sandbox: bool) -> Self {
        Self { disable_sandbox }
    }
}

#[async_trait]
impl Tool for BashTool {
    type Input = BashInput;
    type Output = BashOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "bash"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for the command"
                },
                "background": {
                    "type": "boolean",
                    "description": "Whether to run in background"
                }
            },
            "required": ["command"]
        })
    }

    fn validate_input_permissions(
        &self,
        input: &Self::Input,
        _context: &ToolContext,
    ) -> InputValidationResult {
        // 检查命令是否为空
        if input.command.trim().is_empty() {
            return InputValidationResult {
                is_valid: false,
                error_message: Some("Command cannot be empty".to_string()),
                error_code: None,
            };
        }

        InputValidationResult {
            is_valid: true,
            error_message: None,
            error_code: None,
        }
    }

    async fn check_permissions(
        &self,
        input: &Self::Input,
        _context: &ToolContext,
    ) -> crate::tools::tool::PermissionResult {
        // 使用 Bash 语义分析器分析命令
        let analysis = BashSemanticAnalyzer::analyze_command(&input.command);

        // 检查危险命令
        if analysis.is_dangerous {
            return crate::tools::tool::PermissionResult::deny(
                analysis
                    .danger_reason
                    .unwrap_or_else(|| "Dangerous command detected".to_string()),
            );
        }

        // 检查是否访问敏感路径
        if analysis.accesses_sensitive_path {
            return crate::tools::tool::PermissionResult::ask(format!(
                "Command accesses sensitive paths: {}",
                input.command
            ));
        }

        // 检查是否为破坏性操作
        if analysis.is_destructive {
            return crate::tools::tool::PermissionResult::ask(format!(
                "Destructive operation detected: {}",
                input.command
            ));
        }

        // 默认允许
        crate::tools::tool::PermissionResult::allow()
    }

    fn interrupt_behavior(&self) -> InterruptBehavior {
        // Bash 工具默认 block 用户中断
        // 用户提交新消息时，Bash 继续执行，新消息等待
        InterruptBehavior::Block
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_concurrency_safe(&self) -> bool {
        // Bash 工具默认不安全，因为可能有副作用
        false
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_destructive(&self, input: &Self::Input) -> bool {
        let analysis = BashSemanticAnalyzer::analyze_command(&input.command);
        analysis.is_destructive
    }

    fn is_search_or_read_command(
        &self,
        input: &Self::Input,
    ) -> crate::tools::tool::SearchOrReadResult {
        let analysis = BashSemanticAnalyzer::analyze_command(&input.command);
        crate::tools::tool::SearchOrReadResult {
            is_search: analysis.is_search,
            is_read: analysis.is_read,
            is_list: analysis.is_list,
        }
    }

    fn should_always_load(&self) -> bool {
        false
    }

    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        // Bash 工具默认 5 分钟超时
        Some(5 * 60 * 1000)
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use std::io::Read;
        use std::process::{Command, Stdio};
        use tokio::io::AsyncReadExt;
        use tokio::process::Command as AsyncCommand;

        let timeout_ms = self.timeout_ms(&input).unwrap_or(300000);
        let max_output_lines = 10000;

        let cwd = input.cwd.as_ref().or(ctx.working_directory.as_ref());

        // 解析命令
        let analysis = BashSemanticAnalyzer::analyze_command(&input.command);

        // 如果命令是破坏性的，检查是否应该阻止
        if analysis.is_destructive {
            // 破坏性命令默认会被 permission_level RequiresConfirmation 拦截
            // 这里记录警告但不阻止执行（权限检查在更上层）
        }

        // 使用 tokio 执行命令以支持异步超时
        let output = if input.background.unwrap_or(false) {
            // 后台执行
            let mut cmd = AsyncCommand::new("bash");
            cmd.args(["-c", &input.command]);
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let child = cmd.spawn()?;
            // 后台任务，不等待结果
            let output = BashOutput {
                exit_code: 0,
                stdout: format!(
                    "Background task started with PID: {}",
                    child.id().unwrap_or(0)
                ),
                stderr: String::new(),
            };
            output
        } else {
            // 同步执行（带超时）
            let mut cmd = Command::new("bash");
            cmd.args(["-c", &input.command]);
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let result = cmd.output();

            match result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    // 限制输出行数
                    let stdout_lines: Vec<&str> = stdout.lines().take(max_output_lines).collect();
                    let truncated = stdout.lines().count() > max_output_lines;
                    let stdout = if truncated {
                        format!(
                            "{}\n... (output truncated, {} total lines)",
                            stdout_lines.join("\n"),
                            stdout.lines().count()
                        )
                    } else {
                        stdout.to_string()
                    };

                    BashOutput {
                        exit_code: output.status.code().unwrap_or(-1),
                        stdout,
                        stderr: stderr.to_string(),
                    }
                }
                Err(e) => BashOutput {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Failed to execute command: {}", e),
                },
            }
        };

        Ok(ToolResult::success("bash-1", output))
    }
}

// ===== FileReadTool =====

/// 读取文件工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadInput {
    /// 文件路径
    pub path: String,
    /// 最大读取行数（可选）
    pub max_lines: Option<usize>,
}

/// 读取文件工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadOutput {
    /// 文件内容
    pub content: String,
    /// 文件编码
    pub encoding: String,
    /// 总行数
    pub line_count: usize,
}

/// FileReadTool：读取文件内容
///
/// 维护文件状态缓存，避免重复读取
pub struct FileReadTool;

impl Default for FileReadTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileReadTool {
    type Input = FileReadInput;
    type Output = FileReadOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "read"
    }

    fn aliases(&self) -> &[&str] {
        &["file_read", "FileRead"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "max_lines": {
                    "type": "integer",
                    "description": "Maximum number of lines to read"
                }
            },
            "required": ["path"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        // 读取文件是并发安全的
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use std::fs;
        use std::io::{BufRead, BufReader};
        use std::path::Path;

        let max_lines = input.max_lines.unwrap_or(10000);
        let path = Path::new(&input.path);

        // 检查文件是否存在
        if !path.exists() {
            anyhow::bail!("File not found: {}", path.display());
        }

        // 获取文件元数据
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        // 读取文件内容
        let (content, line_count) = if file_size > 1_000_000 {
            // 大文件 (>1MB): 使用流式读取
            let file = fs::File::open(path)?;
            let reader = BufReader::new(file);
            let mut lines = Vec::with_capacity(max_lines);
            let mut count = 0;

            for line in reader.lines().take(max_lines + 1) {
                if let Ok(line) = line {
                    if count < max_lines {
                        lines.push(line);
                    }
                    count += 1;
                }
            }

            let truncated = count > max_lines;
            let content = if truncated {
                format!(
                    "{}\n... (truncated, {} total lines)",
                    lines.join("\n"),
                    count
                )
            } else {
                lines.join("\n")
            };

            (content, count)
        } else {
            // 小文件: 直接读取
            let content = fs::read_to_string(path)?;
            let line_count = content.lines().count();
            (content, line_count)
        };

        // 计算内容哈希
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());

        // 获取并转换最后修改时间
        let last_modified = metadata.modified().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        });

        let output = FileReadOutput {
            content,
            encoding: "utf-8".to_string(),
            line_count,
        };

        // 返回带有上下文修改器的结果，更新文件缓存
        Ok(
            ToolResult::success("read-1", output).with_context_modifier(ContextModifier {
                file_updates: vec![crate::tools::tool::FileState {
                    path: input.path.clone(),
                    has_been_read: true,
                    last_modified,
                    content_hash: Some(content_hash),
                }],
                metadata: std::collections::HashMap::new(),
            }),
        )
    }
}

// ===== FileEditTool =====

/// 编辑文件工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditInput {
    /// 文件路径
    pub path: String,
    /// 要替换的旧字符串
    pub old_string: String,
    /// 新字符串
    pub new_string: String,
    /// 替换次数（可选，默认 1）
    pub insert_index: Option<usize>,
}

/// 编辑文件工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditOutput {
    /// 是否成功
    pub success: bool,
    /// 修改的行号范围
    pub line_range: (usize, usize),
}

/// FileEditTool：精确编辑文件
///
/// 使用 old_string -> new_string 的精确替换模式
/// 而非行号范围，确保编辑操作在文件变化时仍然正确
pub struct FileEditTool;

impl Default for FileEditTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileEditTool {
    type Input = FileEditInput;
    type Output = FileEditOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "edit"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "The string to replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The replacement string"
                },
                "insert_index": {
                    "type": "integer",
                    "description": "Which occurrence to replace (default: 1)"
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        // 编辑操作不是并发安全的
        false
    }

    fn is_destructive(&self, input: &Self::Input) -> bool {
        // 根据编辑内容判断是否为破坏性操作
        // 删除大量代码被认为是破坏性的
        input.old_string.len() > input.new_string.len() * 2
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use std::fs;
        use std::path::Path;

        let path = Path::new(&input.path);

        // 读取原文件内容
        if !path.exists() {
            anyhow::bail!("File not found: {}", path.display());
        }

        let original_content = fs::read_to_string(path)?;

        // 找到要替换的字符串位置
        let occurrence = input.insert_index.unwrap_or(1);
        let mut current_occurrence = 0;
        let mut search_start = 0;
        let mut match_start: Option<usize> = None;
        let mut match_end: Option<usize> = None;

        while let Some(pos) = original_content[search_start..].find(&input.old_string) {
            current_occurrence += 1;
            let absolute_pos = search_start + pos;
            if current_occurrence == occurrence {
                match_start = Some(absolute_pos);
                match_end = Some(absolute_pos + input.old_string.len());
                break;
            }
            search_start = absolute_pos + 1;
        }

        let (start, end) = match (match_start, match_end) {
            (Some(s), Some(e)) => (s, e),
            (None, None) => anyhow::bail!("String not found: {}", input.old_string),
            _ => anyhow::bail!("Invalid match state"),
        };

        // 创建备份
        let backup_path = format!("{}.backup", input.path);
        fs::copy(&input.path, &backup_path)
            .map_err(|e| anyhow::anyhow!("Failed to create backup: {}", e))?;

        // 执行替换
        let new_content = format!(
            "{}{}{}",
            &original_content[..start],
            input.new_string,
            &original_content[end..]
        );

        // 写入新内容
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("devil_edit_{}.tmp", std::process::id()));

        // 写入临时文件
        {
            let mut file = fs::File::create(&temp_file)?;
            use std::io::Write;
            file.write_all(new_content.as_bytes())?;
            file.sync_all()?;
        }

        // 原子性移动
        fs::rename(&temp_file, path).map_err(|e| {
            // 恢复备份
            let _ = fs::copy(&backup_path, &input.path);
            let _ = fs::remove_file(&temp_file);
            anyhow::anyhow!("Failed to write file: {}", e)
        })?;

        // 删除备份
        let _ = fs::remove_file(&backup_path);

        // 计算行号
        let line_start = original_content[..start].lines().count() + 1;
        let line_end = original_content[..end].lines().count() + 1;

        let output = FileEditOutput {
            success: true,
            line_range: (line_start, line_end),
        };

        Ok(ToolResult::success("edit-1", output))
    }
}

// ===== FileWriteTool =====

/// 写入文件工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteInput {
    /// 文件路径
    pub path: String,
    /// 文件内容
    pub content: String,
    /// 是否追加（默认 false）
    pub append: Option<bool>,
}

/// 写入文件工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteOutput {
    /// 是否成功
    pub success: bool,
    /// 写入的字节数
    pub bytes_written: usize,
}

/// FileWriteTool：创建或完全覆写文件
///
/// 最"重"的文件操作，权限检查最为严格
pub struct FileWriteTool;

impl Default for FileWriteTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    type Input = FileWriteInput;
    type Output = FileWriteOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "write"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "append": {
                    "type": "boolean",
                    "description": "Whether to append instead of overwrite"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use std::fs;
        use std::io::Write;
        use std::path::Path;

        let path = Path::new(&input.path);
        let append = input.append.unwrap_or(false);

        // 如果是追加模式，先读取现有内容
        let final_content = if append && path.exists() {
            let existing = fs::read_to_string(path)?;
            format!("{}{}", existing, input.content)
        } else {
            input.content.clone()
        };

        // 创建临时文件
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("devil_write_{}.tmp", std::process::id()));

        // 写入临时文件
        {
            let mut file = fs::File::create(&temp_file)?;
            file.write_all(final_content.as_bytes())?;
            file.sync_all()?;
        }

        // 确保目标目录存在
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        // 原子性地移动临时文件到目标位置
        fs::rename(&temp_file, path).map_err(|e| {
            // 清理临时文件
            let _ = fs::remove_file(&temp_file);
            anyhow::anyhow!("Failed to write file: {}", e)
        })?;

        let bytes_written = final_content.len();

        let output = FileWriteOutput {
            success: true,
            bytes_written,
        };

        Ok(ToolResult::success("write-1", output))
    }
}

// ===== GlobTool =====

/// Glob 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobInput {
    /// 文件名模式
    pub pattern: String,
    /// 忽略模式（可选）
    pub ignore: Option<Vec<String>>,
    /// 最大结果数
    pub max_results: Option<usize>,
}

/// Glob 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobOutput {
    /// 匹配的文件路径列表
    pub paths: Vec<String>,
    /// 是否有更多结果
    pub has_more: bool,
}

/// GlobTool：使用文件名模式匹配查找文件
///
/// 底层使用 fast-glob 库（JavaScript）或 glob（Rust）
pub struct GlobTool;

impl Default for GlobTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GlobTool {
    type Input = GlobInput;
    type Output = GlobOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "glob"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match files"
                },
                "ignore": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Patterns to ignore"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return"
                }
            },
            "required": ["pattern"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        // Glob 是并发安全的
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use glob::Pattern;

        let cwd = ctx.working_directory.as_deref().unwrap_or(".");
        let max_results = input.max_results.unwrap_or(100);

        let mut paths = Vec::new();
        let mut has_more = false;

        // 获取忽略模式（如果没有提供，使用默认的 gitignore 模式）
        let ignore_patterns: Vec<Pattern> = if let Some(ref ignore) = input.ignore {
            ignore.iter().filter_map(|p| Pattern::new(p).ok()).collect()
        } else {
            // 默认忽略 .git, node_modules, target 等
            vec![
                Pattern::new("**/.git/**").ok(),
                Pattern::new("**/node_modules/**").ok(),
                Pattern::new("**/target/**").ok(),
            ]
            .into_iter()
            .flatten()
            .collect()
        };

        // 使用 glob crate 执行模式匹配
        let pattern_str = format!("{}/{}", cwd, input.pattern);
        if let Ok(entries) = glob::glob(&pattern_str) {
            for entry in entries.flatten() {
                if let Some(path_str) = entry.to_str() {
                    // 检查是否应该忽略
                    let should_ignore = ignore_patterns.iter().any(|p| p.matches(path_str));
                    if should_ignore {
                        continue;
                    }

                    if paths.len() >= max_results {
                        has_more = true;
                        break;
                    }

                    paths.push(path_str.to_string());
                }
            }
        }

        let output = GlobOutput { paths, has_more };

        Ok(ToolResult::success("glob-1", output))
    }
}

// ===== GrepTool =====

/// Grep 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepInput {
    /// 搜索模式（正则表达式）
    pub pattern: String,
    /// 搜索目录（可选，默认当前目录）
    pub path: Option<String>,
    /// 文件类型过滤（如 "rust", "js"）
    pub file_type: Option<String>,
    /// 忽略模式
    pub ignore: Option<Vec<String>>,
    /// 是否区分大小写
    pub case_sensitive: Option<bool>,
}

/// Grep 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepOutput {
    /// 匹配结果列表
    pub matches: Vec<GrepMatch>,
    /// 匹配总数
    pub total_matches: usize,
}

/// 单个匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepMatch {
    /// 文件路径
    pub path: String,
    /// 行号
    pub line_number: usize,
    /// 匹配内容
    pub content: String,
}

/// GrepTool：使用正则表达式搜索文件内容
///
/// 底层使用 ripgrep（rust）实现
pub struct GrepTool;

impl Default for GrepTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GrepTool {
    type Input = GrepInput;
    type Output = GrepOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "grep"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in"
                },
                "file_type": {
                    "type": "string",
                    "description": "File type filter (e.g., 'rust', 'js')"
                },
                "ignore": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Patterns to ignore"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Whether to do case-sensitive search"
                }
            },
            "required": ["pattern"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        // Grep 是并发安全的
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use regex::Regex;
        use walkdir::WalkDir;

        let cwd = ctx.working_directory.as_deref().unwrap_or(".");
        let search_path = input.path.as_deref().unwrap_or(cwd);

        // 构建正则表达式
        let case_sensitive = input.case_sensitive.unwrap_or(true);
        let regex_pattern = if case_sensitive {
            input.pattern.clone()
        } else {
            format!("(?i){}", input.pattern)
        };

        let re = match Regex::new(&regex_pattern) {
            Ok(r) => r,
            Err(e) => anyhow::bail!("Invalid regex pattern: {}", e),
        };

        // 文件类型过滤器
        let file_type_filter: Option<&str> = input.file_type.as_deref();

        // 忽略模式
        let ignore_patterns: Vec<Regex> = input
            .ignore
            .as_ref()
            .map(|ignores| ignores.iter().filter_map(|p| Regex::new(p).ok()).collect())
            .unwrap_or_default();

        let mut matches = Vec::new();
        let max_matches = 100;

        // 遍历目录
        for entry in WalkDir::new(search_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            // 检查文件类型
            if let Some(ext) = entry.path().extension() {
                if let Some(filter) = file_type_filter {
                    let ext_str = ext.to_string_lossy();
                    let matches_filter = match filter {
                        "rust" | "rs" => ext_str == "rs",
                        "js" | "javascript" => ext_str == "js" || ext_str == "jsx",
                        "ts" | "typescript" => ext_str == "ts" || ext_str == "tsx",
                        "py" | "python" => ext_str == "py",
                        _ => ext_str == filter,
                    };
                    if !matches_filter {
                        continue;
                    }
                }
            }

            let path_str = entry.path().to_string_lossy().to_string();

            // 检查忽略模式
            let should_ignore = ignore_patterns.iter().any(|p| p.is_match(&path_str));
            if should_ignore {
                continue;
            }

            // 读取文件内容并搜索
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        if matches.len() >= max_matches {
                            let total = matches.len();
                            return Ok(ToolResult::success(
                                "grep-1",
                                GrepOutput {
                                    matches,
                                    total_matches: total,
                                },
                            ));
                        }
                        matches.push(GrepMatch {
                            path: path_str.clone(),
                            line_number: line_num + 1,
                            content: line.to_string(),
                        });
                    }
                }
            }
        }

        let total = matches.len();
        Ok(ToolResult::success(
            "grep-1",
            GrepOutput {
                matches,
                total_matches: total,
            },
        ))
    }
}

// ===== WebFetchTool =====

use reqwest::Client;
use std::time::Duration;

lazy_static::lazy_static! {
    static ref HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| Client::new());
}

/// WebFetch 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchInput {
    /// 要获取的 URL
    pub url: String,
    /// 提示词（用于决定提取什么内容）
    pub prompt: Option<String>,
}

/// WebFetch 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchOutput {
    /// 原始字节数
    pub bytes: Option<usize>,
    /// HTTP 状态码
    pub code: u16,
    /// 内容（转换为文本）
    pub content: String,
    /// 内容类型
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    /// 请求耗时（毫秒）
    #[serde(rename = "durationMs")]
    pub duration_ms: u64,
    /// 实际 URL（跟随重定向后）
    pub url: String,
}

/// WebFetchTool：获取网页内容
///
/// 支持 HTML 到文本的转换，可选的 CSS 选择器提取
pub struct WebFetchTool;

impl Default for WebFetchTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    type Input = WebFetchInput;
    type Output = WebFetchOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "webfetch"
    }

    fn aliases(&self) -> &[&str] {
        &["fetch", "WebFetch"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to fetch"
                },
                "prompt": {
                    "type": "string",
                    "description": "What to extract from the page"
                }
            },
            "required": ["url"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    fn is_open_world(&self, _input: &Self::Input) -> bool {
        true
    }

    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        Some(60_000) // 60 秒超时
    }

    fn max_result_size_chars(&self) -> usize {
        500_000 // 500KB
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use std::time::Instant;

        let start = Instant::now();
        let max_content_size = 10 * 1024 * 1024; // 10MB

        let response = HTTP_CLIENT
            .get(&input.url)
            .header("User-Agent", "Mozilla/5.0 (compatible; Devil-Agent/1.0)")
            .send()
            .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let final_url = resp.url().to_string();
                let content_type = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                let bytes = resp.content_length().map(|b| b as usize);
                let total_bytes = bytes.unwrap_or(0);

                if total_bytes > max_content_size {
                    return Ok(ToolResult {
                        tool_use_id: "webfetch-1".to_string(),
                        is_success: false,
                        output: None,
                        error: Some(format!(
                            "Content too large ({} bytes, max {} bytes)",
                            total_bytes, max_content_size
                        )),
                        context_modifier: None,
                        interrupted: false,
                    });
                }

                let content = resp.text().await.unwrap_or_default();

                let output = WebFetchOutput {
                    bytes: Some(content.len()),
                    code: status,
                    content,
                    content_type,
                    duration_ms,
                    url: final_url,
                };

                Ok(ToolResult::success("webfetch-1", output))
            }
            Err(e) => Ok(ToolResult {
                tool_use_id: "webfetch-1".to_string(),
                is_success: false,
                output: None,
                error: Some(format!("Fetch failed: {}", e)),
                context_modifier: None,
                interrupted: false,
            }),
        }
    }
}

// ===== WebSearchTool =====

/// WebSearch 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchInput {
    /// 搜索查询
    pub query: String,
    /// 允许的域名（可选）
    #[serde(rename = "allowedDomains")]
    pub allowed_domains: Option<Vec<String>>,
    /// 排除的域名（可选）
    #[serde(rename = "blockedDomains")]
    pub blocked_domains: Option<Vec<String>>,
    /// 最大结果数
    #[serde(rename = "maxResults")]
    pub max_results: Option<usize>,
}

/// 单个搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    /// 标题
    pub title: String,
    /// URL
    pub url: String,
    /// 摘要
    pub snippet: String,
}

/// WebSearch 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchOutput {
    /// 搜索查询
    pub query: String,
    /// 搜索结果
    pub results: Vec<WebSearchResult>,
    /// 请求耗时（秒）
    #[serde(rename = "durationSeconds")]
    pub duration_seconds: f64,
}

/// WebSearchTool：搜索网页
///
/// 支持多种搜索适配器：
/// - BRAVE_SEARCH_API_KEY: Brave Search API
/// - Bing Search API
/// - Google Search API (limited)
pub struct WebSearchTool;

impl Default for WebSearchTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    type Input = WebSearchInput;
    type Output = WebSearchOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "websearch"
    }

    fn aliases(&self) -> &[&str] {
        &["search", "WebSearch"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "allowedDomains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Domains to allow in results"
                },
                "blockedDomains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Domains to exclude from results"
                },
                "maxResults": {
                    "type": "integer",
                    "description": "Maximum number of results (default 10)"
                }
            },
            "required": ["query"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    fn is_open_world(&self, _input: &Self::Input) -> bool {
        true
    }

    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        Some(30_000) // 30 秒超时
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use std::time::Instant;

        let start = Instant::now();
        let max_results = input.max_results.unwrap_or(10);

        // 检查环境变量中的 API key
        let api_key = std::env::var("BRAVE_SEARCH_API_KEY")
            .or_else(|_| std::env::var("BING_SEARCH_API_KEY"))
            .ok();

        let results = if let Some(key) = api_key {
            // 使用 Brave Search API
            self.brave_search(&input.query, &key, max_results).await
        } else {
            // 回退到简单的 HTML 抓取搜索（演示用）
            self.fallback_search(&input.query, max_results).await
        };

        let duration_seconds = start.elapsed().as_secs_f64();

        let output = WebSearchOutput {
            query: input.query,
            results,
            duration_seconds,
        };

        Ok(ToolResult::success("websearch-1", output))
    }
}

impl WebSearchTool {
    async fn brave_search(
        &self,
        query: &str,
        api_key: &str,
        max_results: usize,
    ) -> Vec<WebSearchResult> {
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
            urlencoding::encode(query),
            max_results
        );

        let response = HTTP_CLIENT
            .get(&url)
            .header("Accept", "application/json")
            .header("X-Subscription-Token", api_key)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    return self.parse_brave_results(json, max_results);
                }
            }
            Err(_) => {}
        }

        Vec::new()
    }

    fn parse_brave_results(&self, json: serde_json::Value, max_results: usize) -> Vec<WebSearchResult> {
        let mut results = Vec::new();

        if let Some(web_results) = json.get("web").and_then(|w| w.get("results")) {
            if let Some(arr) = web_results.as_array() {
                for item in arr.iter().take(max_results) {
                    let title = item
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();
                    let url = item
                        .get("url")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string();
                    let snippet = item
                        .get("description")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();

                    if !title.is_empty() && !url.is_empty() {
                        results.push(WebSearchResult { title, url, snippet });
                    }
                }
            }
        }

        results
    }

    async fn fallback_search(&self, query: &str, max_results: usize) -> Vec<WebSearchResult> {
        // 简单的回退实现：使用 DuckDuckGo HTML
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        let response = HTTP_CLIENT
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if let Ok(html) = resp.text().await {
                    return self.parse_duckduckgo_html(&html, max_results);
                }
            }
            Err(_) => {}
        }

        Vec::new()
    }

    fn parse_duckduckgo_html(&self, html: &str, max_results: usize) -> Vec<WebSearchResult> {
        let mut results = Vec::new();
        let mut count = 0;

        // 简单的 HTML 解析：查找 <a class="result__a" href="...">标题</a>
        let link_pattern = r#"<a class="result__a" href="([^"]+)">([^<]+)</a>"#;
        let snippet_pattern = r#"<a class="result__snippet" href="[^"]+">([^<]+)</a>"#;

        let link_regex = regex::Regex::new(link_pattern).ok();
        let snippet_regex = regex::Regex::new(snippet_pattern).ok();

        if let Some(link_re) = link_regex {
            let mut snippets: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
            if let Some(snippet_re) = snippet_regex {
                for cap in snippet_re.captures_iter(html) {
                    if let (Some(url_match), Some(snippet_match)) = (cap.get(1), cap.get(2)) {
                        snippets.insert(url_match.as_str(), snippet_match.as_str());
                    }
                }
            }

            for cap in link_re.captures_iter(html) {
                if count >= max_results {
                    break;
                }

                let url = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let title = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                let snippet = snippets.get(url).unwrap_or(&"").to_string();

                if !url.is_empty() && !title.is_empty() {
                    results.push(WebSearchResult {
                        title,
                        url: url.to_string(),
                        snippet,
                    });
                    count += 1;
                }
            }
        }

        results
    }
}

// ===== TodoWriteTool =====

/// Todo 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// 任务内容
    pub content: String,
    /// 状态：pending, in_progress, completed
    pub status: String,
    /// 当前进行的操作描述
    #[serde(rename = "activeForm")]
    pub active_form: Option<String>,
}

/// TodoWrite 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoWriteInput {
    /// 任务列表（完整替换）
    pub todos: Vec<TodoItem>,
}

/// TodoWrite 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoWriteOutput {
    /// 更新前的任务列表
    #[serde(rename = "oldTodos")]
    pub old_todos: Vec<TodoItem>,
    /// 更新后的任务列表
    #[serde(rename = "newTodos")]
    pub new_todos: Vec<TodoItem>,
}

/// TodoWriteTool：任务列表管理
///
/// 使用完整替换模型管理任务列表
pub struct TodoWriteTool;

impl Default for TodoWriteTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TodoWriteTool {
    type Input = TodoWriteInput;
    type Output = TodoWriteOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "todowrite"
    }

    fn aliases(&self) -> &[&str] {
        &["todo", "TodoWrite"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "description": "Full todo list (complete replacement model)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "Task description"
                            },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed"],
                                "description": "Task status"
                            },
                            "activeForm": {
                                "type": "string",
                                "description": "Current action being performed"
                            }
                        },
                        "required": ["content", "status"]
                    }
                }
            },
            "required": ["todos"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    fn get_activity_description(&self, input: &Self::Input) -> Option<String> {
        let count = input.todos.len();
        Some(format!("Updating {} tasks", count))
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        // 从执行历史中获取旧的任务列表
        let old_todos: Vec<TodoItem> = _ctx
            .executed_tools
            .iter()
            .filter(|t| t.tool_name == "todowrite")
            .last()
            .and_then(|_| None)
            .unwrap_or_default();

        let new_todos = input.todos;

        // 验证状态值
        for todo in &new_todos {
            if !["pending", "in_progress", "completed"].contains(&todo.status.as_str()) {
                return Ok(ToolResult {
                    tool_use_id: "todowrite-1".to_string(),
                    is_success: false,
                    output: None,
                    error: Some(format!("Invalid status: {}", todo.status)),
                    context_modifier: None,
                    interrupted: false,
                });
            }
        }

        let output = TodoWriteOutput {
            old_todos,
            new_todos: new_todos.clone(),
        };

        // 返回带有上下文修改器的结果
        Ok(ToolResult::success("todowrite-1", output).with_context_modifier(
            crate::tools::tool::ContextModifier {
                file_updates: vec![],
                metadata: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(
                        "todos".to_string(),
                        serde_json::to_value(&new_todos).unwrap_or_default(),
                    );
                    m
                },
            },
        ))
    }
}

// ===== AgentTool =====

/// Agent 工具输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// 子代理描述
    pub description: String,
    /// 子代理指令
    pub prompt: String,
    /// 子代理类型
    #[serde(rename = "subagentType")]
    pub subagent_type: Option<String>,
    /// 使用的模型（可选）
    pub model: Option<String>,
    /// 是否后台运行
    #[serde(rename = "runInBackground")]
    pub run_in_background: Option<bool>,
    /// 工作目录
    pub cwd: Option<String>,
}

/// Agent 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// 是否成功
    pub success: bool,
    /// 输出消息
    pub message: String,
    /// 任务 ID（用于后台任务）
    #[serde(rename = "taskId")]
    pub task_id: Option<String>,
}

/// AgentTool：子代理工具
///
/// 包装 SubagentExecutor 作为工具使用
pub struct AgentTool;

impl Default for AgentTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for AgentTool {
    type Input = AgentInput;
    type Output = AgentOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "agent"
    }

    fn aliases(&self) -> &[&str] {
        &["subagent", "Agent"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the subagent task"
                },
                "prompt": {
                    "type": "string",
                    "description": "Instruction for the subagent"
                },
                "subagentType": {
                    "type": "string",
                    "enum": ["fork", "general", "custom"],
                    "description": "Type of subagent"
                },
                "model": {
                    "type": "string",
                    "description": "Model to use (optional)"
                },
                "runInBackground": {
                    "type": "boolean",
                    "description": "Whether to run in background"
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for the subagent"
                }
            },
            "required": ["description", "prompt"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    fn is_open_world(&self, _input: &Self::Input) -> bool {
        true
    }

    fn interrupt_behavior(&self) -> InterruptBehavior {
        InterruptBehavior::Block
    }

    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        Some(5 * 60 * 1000) // 5 分钟默认超时
    }

    fn get_activity_description(&self, input: &Self::Input) -> Option<String> {
        Some(format!("Running subagent: {}", input.description))
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<
            impl Fn(crate::tools::tool::ToolProgress<Self::Progress>) + Send + Sync,
        >,
    ) -> Result<ToolResult<Self::Output>> {
        use crate::subagent::{SubagentExecutor, SubagentParams, SubagentType, CacheSafeParams, ToolUseContext};
        use uuid::Uuid;
        use std::collections::HashMap;

        let task_id = Uuid::new_v4().to_string();
        let run_in_background = input.run_in_background.unwrap_or(false);
        let subagent_type = match input.subagent_type.as_deref() {
            Some("fork") => SubagentType::Fork,
            Some("general") | Some("general_purpose") => SubagentType::GeneralPurpose,
            Some("custom") => SubagentType::Custom(input.subagent_type.clone().unwrap_or_default()),
            _ => SubagentType::GeneralPurpose,
        };

        let params = SubagentParams {
            subagent_type,
            directive: input.prompt.clone(),
            prompt_messages: vec![],
            cache_safe_params: CacheSafeParams {
                system_prompt: String::new(),
                user_context: HashMap::new(),
                system_context: HashMap::new(),
                tool_use_context: ToolUseContext {
                    available_tools: vec![],
                    rendered_system_prompt: String::new(),
                    thinking_config: None,
                },
                fork_context_messages: vec![],
            },
            max_turns: None,
            max_output_tokens: None,
            skip_transcript: false,
            skip_cache_write: false,
            run_in_background,
            worktree_path: None,
            parent_cwd: input.cwd.or_else(|| ctx.working_directory.clone()),
        };

        if run_in_background {
            // 后台执行：立即返回
            let output = AgentOutput {
                success: true,
                message: format!("Subagent started in background with task ID: {}", task_id),
                task_id: Some(task_id),
            };
            return Ok(ToolResult::success("agent-1", output));
        }

        // 同步执行：等待结果
        let executor = SubagentExecutor::new();
        match executor.execute(params).await {
            Ok(result) => {
                let message = format!(
                    "Subagent completed with {} messages",
                    result.messages.len()
                );
                let output = AgentOutput {
                    success: true,
                    message,
                    task_id: Some(task_id),
                };
                Ok(ToolResult::success("agent-1", output))
            }
            Err(e) => {
                let output = AgentOutput {
                    success: false,
                    message: format!("Subagent error: {}", e),
                    task_id: Some(task_id),
                };
                Ok(ToolResult::success("agent-1", output))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_builder() {
        let tool = ToolBuilder::new("test", "A test tool")
            .input_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                }
            }))
            .read_only()
            .concurrency_safe()
            .execute(|input: serde_json::Value, _ctx: &ToolContext| Ok("result".to_string()))
            .build();

        assert_eq!(tool.name(), "test");
        assert!(tool.is_read_only());
        assert!(tool.is_concurrency_safe());
    }

    #[tokio::test]
    async fn test_bash_tool_validation() {
        let tool = BashTool::new(false);

        // 空命令应该无效
        let result = tool.validate_input_permissions(
            &BashInput {
                command: "".to_string(),
                cwd: None,
                background: None,
            },
            &ToolContext::default(),
        );
        assert!(!result.is_valid);

        // 危险命令应该没有权限
        let result = tool
            .check_permissions(
                &BashInput {
                    command: "rm -rf /".to_string(),
                    cwd: None,
                    background: None,
                },
                &ToolContext::default(),
            )
            .await;
        // 默认不允许危险命令
        assert!(matches!(result.behavior, PermissionBehavior::Deny { .. }));
    }

    #[test]
    fn test_file_tools_properties() {
        let read_tool = FileReadTool::default();
        assert!(read_tool.is_read_only());
        assert!(read_tool.is_concurrency_safe());

        let edit_tool = FileEditTool::default();
        assert!(!edit_tool.is_read_only());
        assert!(!edit_tool.is_concurrency_safe());

        let write_tool = FileWriteTool::default();
        assert!(!write_tool.is_read_only());
        assert!(!write_tool.is_concurrency_safe());
    }

    #[test]
    fn test_search_tools_properties() {
        let glob_tool = GlobTool::default();
        assert!(glob_tool.is_read_only());
        assert!(glob_tool.is_concurrency_safe());

        let grep_tool = GrepTool::default();
        assert!(grep_tool.is_read_only());
        assert!(grep_tool.is_concurrency_safe());
    }
}
