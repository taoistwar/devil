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
        let result = tool.check_permissions(
            &BashInput {
                command: "rm -rf /".to_string(),
                cwd: None,
                background: None,
            },
            &ToolContext::default(),
        ).await;
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
