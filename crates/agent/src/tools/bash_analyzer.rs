//! Bash 命令分析器模块
//!
//! 基于 Claude Code 的 bash/ast.js 和 commandSemantics.ts 实现：
//! - 命令 AST 解析
//! - 语义分析（搜索/读取/列表操作检测）
//! - 静默命令检测
//! - 自动后台检测

use lazy_static::lazy_static;
use std::collections::HashSet;

/// Bash 搜索结果/读取结果
#[derive(Debug, Clone, Default)]
pub struct BashSemanticResult {
    /// 是否为搜索操作 (grep, find, etc.)
    pub is_search: bool,
    /// 是否为读取操作 (cat, head, tail, etc.)
    pub is_read: bool,
    /// 是否为列表操作 (ls, tree, du, etc.)
    pub is_list: bool,
}

impl BashSemanticResult {
    /// 检查是否为中性命令（可折叠显示）
    pub fn is_neutral(&self) -> bool {
        self.is_search || self.is_read || self.is_list
    }
}

lazy_static! {
    /// 搜索命令集合（用于可折叠显示）
    static ref BASH_SEARCH_COMMANDS: HashSet<&'static str> = HashSet::from([
        "find", "grep", "rg", "ag", "ack", "locate", "which", "whereis",
    ]);

    /// 读取/查看命令集合（用于可折叠显示）
    static ref BASH_READ_COMMANDS: HashSet<&'static str> = HashSet::from([
        "cat", "head", "tail", "less", "more",
        // 分析命令
        "wc", "stat", "file", "strings",
        // 数据处理 - 常用于管道中解析/转换文件内容
        "jq", "awk", "cut", "sort", "uniq", "tr",
    ]);

    /// 列表命令集合（用于可折叠显示）
    static ref BASH_LIST_COMMANDS: HashSet<&'static str> = HashSet::from([
        "ls", "tree", "du", "df", "free",
    ]);

    /// 语义中性命令（不影响搜索/读取/列表判断）
    static ref BASH_SEMANTIC_NEUTRAL_COMMANDS: HashSet<&'static str> = HashSet::from([
        "echo", "printf", "true", "false", "test", "[",
    ]);

    /// 静默命令集合（预期无输出）
    static ref BASH_SILENT_COMMANDS: HashSet<&'static str> = HashSet::from([
        "cp", "mv", "mkdir", "rm", "rmdir", "touch", "chmod", "chown",
        "ln", "pwd", "cd", "pushd", "popd", "dirs", "export", "unset",
        "alias", "unalias", "source", ".", "set", "shopt",
    ]);

    /// 不允许自动后台的命令
    static ref DISALLOWED_AUTO_BACKGROUND_COMMANDS: HashSet<&'static str> = HashSet::from([
        "sleep", // sleep 应在前台运行，除非用户明确后台
    ]);
}

/// 分割命令字符串（简化版本）
///
/// 处理基本的操作符分割：| && || ; > >> <
pub fn split_command_with_operators(command: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '|' => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                // 检查 ||
                if chars.peek() == Some(&'|') {
                    chars.next();
                    parts.push("||".to_string());
                } else {
                    parts.push("|".to_string());
                }
            }
            '&' => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                // 检查 &&
                if chars.peek() == Some(&'&') {
                    chars.next();
                    parts.push("&&".to_string());
                } else {
                    parts.push("&".to_string());
                }
            }
            ';' => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                parts.push(";".to_string());
            }
            '>' => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                // 检查 >>
                if chars.peek() == Some(&'>') {
                    chars.next();
                    parts.push(">>".to_string());
                } else {
                    parts.push(">".to_string());
                }
            }
            '<' => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                parts.push("<".to_string());
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

/// 简化版本的命令分割（按空格）
pub fn split_command_simple(command: &str) -> Vec<String> {
    command.split_whitespace().map(|s| s.to_string()).collect()
}

/// 检查 Bash 命令是否为搜索或读取操作
///
/// 用于 UI 折叠展示
pub fn is_search_or_read_command(command: &str) -> BashSemanticResult {
    let parts = split_command_with_operators(command);

    if parts.is_empty() {
        return BashSemanticResult::default();
    }

    let mut has_search = false;
    let mut has_read = false;
    let mut has_list = false;
    let mut has_non_neutral_command = false;
    let mut skip_next_as_redirect_target = false;

    for part in &parts {
        if skip_next_as_redirect_target {
            skip_next_as_redirect_target = false;
            continue;
        }

        // 跳过重定向目标
        if part == ">" || part == ">>" || part == ">&" {
            skip_next_as_redirect_target = true;
            continue;
        }

        // 跳过操作符
        if part == "||" || part == "&&" || part == "|" || part == ";" {
            continue;
        }

        // 获取基础命令
        let parts_result = split_command_simple(part);
        let base_command = parts_result.first().map(|s| s.as_str()).unwrap_or("");

        if base_command.is_empty() {
            continue;
        }

        // 检查是否为中性命令
        if BASH_SEMANTIC_NEUTRAL_COMMANDS.contains(base_command) {
            continue;
        }

        has_non_neutral_command = true;

        let is_part_search = BASH_SEARCH_COMMANDS.contains(base_command);
        let is_part_read = BASH_READ_COMMANDS.contains(base_command);
        let is_part_list = BASH_LIST_COMMANDS.contains(base_command);

        // 如果遇到非搜索/读取/列表命令，直接返回 false
        if !is_part_search && !is_part_read && !is_part_list {
            return BashSemanticResult::default();
        }

        if is_part_search {
            has_search = true;
        }
        if is_part_read {
            has_read = true;
        }
        if is_part_list {
            has_list = true;
        }
    }

    // 只有中性命令（如 "echo foo"）不被认为是可折叠的
    if !has_non_neutral_command {
        return BashSemanticResult::default();
    }

    BashSemanticResult {
        is_search: has_search,
        is_read: has_read,
        is_list: has_list,
    }
}

/// 检查 Bash 命令是否预期无输出（用于显示 "Done" 而非 "(No output)"）
pub fn is_silent_bash_command(command: &str) -> bool {
    let parts = split_command_with_operators(command);

    if parts.is_empty() {
        return false;
    }

    let mut has_non_fallback_command = false;
    let mut skip_next_as_redirect_target = false;

    for part in &parts {
        if skip_next_as_redirect_target {
            skip_next_as_redirect_target = false;
            continue;
        }

        if part == ">" || part == ">>" || part == ">&" {
            skip_next_as_redirect_target = true;
            continue;
        }

        if part == "||" || part == "&&" || part == "|" || part == ";" {
            continue;
        }

        let parts_result = split_command_simple(part);
        let base_command = parts_result.first().map(|s| s.as_str()).unwrap_or("");

        if base_command.is_empty() {
            continue;
        }

        // 处理 || 后面的中性命令（如 fallback）
        if BASH_SEMANTIC_NEUTRAL_COMMANDS.contains(base_command) {
            continue;
        }

        has_non_fallback_command = true;

        if !BASH_SILENT_COMMANDS.contains(base_command) {
            return false;
        }
    }

    has_non_fallback_command
}

/// 检查命令是否允许自动后台
pub fn is_auto_backgrounding_allowed(command: &str) -> bool {
    let parts = split_command_simple(command);

    if parts.is_empty() {
        return true;
    }

    let base_command = parts.first().map(|s| s.as_str()).unwrap_or("");

    !DISALLOWED_AUTO_BACKGROUND_COMMANDS.contains(base_command)
}

/// 检测被阻塞的 sleep 模式
///
/// 返回说明字符串，如果检测到 standalone sleep 或 sleep 后跟其他命令
/// 用于建议用户使用 Monitor 模式
pub fn detect_blocked_sleep_pattern(command: &str) -> Option<String> {
    let parts = split_command_simple(command);

    if parts.is_empty() {
        return None;
    }

    let first = &parts[0];

    // 检测 sleep N 或 sleep N.N
    let sleep_pattern = regex::Regex::new(r"^sleep\s+(\d+(?:\.\d+)?)$").unwrap();

    if let Some(caps) = sleep_pattern.captures(first) {
        let secs_str = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
        let secs: f64 = secs_str.parse().unwrap_or(0.0);

        // 小于 2 秒的 sleep 是合法的（用于限流、节奏控制）
        if secs < 2.0 {
            return None;
        }

        let rest = parts[1..].join(" ");

        if rest.trim().is_empty() {
            return Some(format!("standalone sleep {}", secs as i32));
        } else {
            return Some(format!(
                "sleep {} followed by: {}",
                secs as i32,
                rest.trim()
            ));
        }
    }

    None
}

/// 解析命令中的文件和路径
///
/// 从命令字符串中提取文件路径，用于权限检查
pub fn extract_paths_from_command(command: &str) -> Vec<String> {
    let mut paths = Vec::new();

    // 简单实现：查找看起来像路径的字符串
    for part in command.split_whitespace() {
        if part.starts_with('/') || part.starts_with("./") || part.starts_with("../") {
            // 排除重定向符号
            if !part.starts_with(">") && !part.starts_with("<") {
                // 去除引号
                let path = part.trim_matches('"').trim_matches('\'');
                paths.push(path.to_string());
            }
        }
    }

    paths
}

/// 检测命令中的危险模式
#[derive(Debug, Clone, Default)]
pub struct DangerDetection {
    /// 是否危险
    pub is_dangerous: bool,
    /// 危险原因
    pub reason: Option<String>,
    /// 危险级别 (1-10)
    pub danger_level: u8,
}

/// 检测命令中的危险模式
pub fn detect_dangerous_patterns(command: &str) -> DangerDetection {
    let dangerous_patterns = [
        ("rm -rf /", "Recursive delete from root", 10),
        ("rm -rf /*", "Recursive delete all", 10),
        (":(){:|:&};:", "Fork bomb", 10),
        ("mkfs", "Format filesystem", 9),
        ("> /dev/sd", "Write to disk device", 9),
        ("dd if=", "Direct disk write", 9),
        ("chmod -R 777 /", "Make all files executable", 8),
        ("chown -R", "Recursive ownership change", 7),
        ("curl.*\\|.*sh", "Download and execute", 7),
        ("wget.*\\|.*sh", "Download and execute", 7),
    ];

    let command_lower = command.to_lowercase();

    for (pattern, reason, level) in &dangerous_patterns {
        if command_lower.contains(&pattern.to_lowercase()) {
            return DangerDetection {
                is_dangerous: true,
                reason: Some(reason.to_string()),
                danger_level: *level,
            };
        }
    }

    DangerDetection::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_command_with_operators() {
        let parts = split_command_with_operators("cat file.txt | grep hello && ls -la");
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "cat file.txt");
        assert_eq!(parts[1], "|");
        assert_eq!(parts[2], "grep hello");
        assert_eq!(parts[3], "&&");
        assert_eq!(parts[4], "ls -la");
    }

    #[test]
    fn test_is_search_or_read_command_search() {
        let result = is_search_or_read_command("grep -r 'pattern' src/");
        assert!(result.is_search);
        assert!(!result.is_read);
        assert!(!result.is_list);
    }

    #[test]
    fn test_is_search_or_read_command_read() {
        let result = is_search_or_read_command("cat file.txt");
        assert!(!result.is_search);
        assert!(result.is_read);
        assert!(!result.is_list);
    }

    #[test]
    fn test_is_search_or_read_command_list() {
        let result = is_search_or_read_command("ls -la");
        assert!(!result.is_search);
        assert!(!result.is_read);
        assert!(result.is_list);
    }

    #[test]
    fn test_is_search_or_read_command_pipe() {
        let result = is_search_or_read_command("find . -name '*.rs' | xargs grep test");
        assert!(result.is_search);
        assert!(!result.is_read);
        assert!(!result.is_list);
    }

    #[test]
    fn test_is_search_or_read_command_non_neutral() {
        // 包含非中性命令（如 git commit），不应被认为是搜索/读取
        let result = is_search_or_read_command("git commit -m 'test'");
        assert!(!result.is_search);
        assert!(!result.is_read);
        assert!(!result.is_list);
    }

    #[test]
    fn test_is_silent_bash_command() {
        assert!(is_silent_bash_command("mkdir -p foo/bar"));
        assert!(is_silent_bash_command("cp file.txt backup/"));
        assert!(is_silent_bash_command("mv old.txt new.txt"));
        assert!(!is_silent_bash_command("cat file.txt"));
        assert!(!is_silent_bash_command("ls -la"));
    }

    #[test]
    fn test_is_auto_backgrounding_allowed() {
        assert!(!is_auto_backgrounding_allowed("sleep 10"));
        assert!(is_auto_backgrounding_allowed("npm install"));
        assert!(is_auto_backgrounding_allowed("cargo build"));
    }

    #[test]
    fn test_detect_blocked_sleep_pattern() {
        let result = detect_blocked_sleep_pattern("sleep 5");
        assert_eq!(result, Some("standalone sleep 5".to_string()));

        let result = detect_blocked_sleep_pattern("sleep 5 && check");
        assert_eq!(result, Some("sleep 5 followed by: check".to_string()));

        let result = detect_blocked_sleep_pattern("sleep 1");
        assert_eq!(result, None); // 小于 2 秒

        let result = detect_blocked_sleep_pattern("npm install");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_paths_from_command() {
        let paths = extract_paths_from_command("cat /etc/hosts ./config.json ../data/file.txt");
        assert!(paths.contains(&"/etc/hosts".to_string()));
        assert!(paths.contains(&"./config.json".to_string()));
        assert!(paths.contains(&"../data/file.txt".to_string()));
    }

    #[test]
    fn test_detect_dangerous_patterns() {
        let result = detect_dangerous_patterns("rm -rf /tmp/test");
        assert!(!result.is_dangerous);

        let result = detect_dangerous_patterns("rm -rf /");
        assert!(result.is_dangerous);
        assert_eq!(result.danger_level, 10);

        let result = detect_dangerous_patterns("mkfs.ext4 /dev/sda1");
        assert!(result.is_dangerous);
        assert_eq!(result.danger_level, 9);
    }
}
