//! Bash 工具权限分析模块
//!
//! 实现 Bash 命令的 AST 解析和语义分析：
//! - 命令分类（搜索/读取/列表/静默/危险）
//! - 路径安全性检查
//! - 危险命令检测
//! - 沙箱模式判断

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Bash 命令分析结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BashCommandAnalysis {
    /// 原始命令
    pub command: String,
    /// 是否为搜索操作 (grep, find, rg, ag, locate, which)
    pub is_search: bool,
    /// 是否为读取操作 (cat, head, tail, less, wc, stat, jq, awk)
    pub is_read: bool,
    /// 是否为列表操作 (ls, tree, du, df)
    pub is_list: bool,
    /// 是否为静默命令 (cp, mv, mkdir, rm, chmod)
    pub is_silent: bool,
    /// 是否为破坏性操作
    pub is_destructive: bool,
    /// 是否访问敏感路径
    pub accesses_sensitive_path: bool,
    /// 是否危险命令
    pub is_dangerous: bool,
    /// 危险原因
    pub danger_reason: Option<String>,
    /// 是否支持沙箱执行
    pub can_sandbox: bool,
    /// 涉及的命令行首
    pub command_prefix: String,
}

/// 路径安全性级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSafetyLevel {
    /// 安全路径（工作目录内）
    Safe,
    /// 警告路径（工作目录外但非系统路径）
    Warning,
    /// 危险路径（系统关键目录）
    Dangerous,
}

/// Bash 语义分析器
pub struct BashSemanticAnalyzer;

impl BashSemanticAnalyzer {
    /// 分析 Bash 命令
    ///
    /// 解析命令并返回分析结果
    pub fn analyze_command(command: &str) -> BashCommandAnalysis {
        let mut analysis = BashCommandAnalysis {
            command: command.to_string(),
            ..Default::default()
        };

        // 提取命令首词
        let command_prefix = Self::extract_command_prefix(command);
        analysis.command_prefix = command_prefix.clone();

        // 命令分类
        analysis.is_search = Self::is_search_command(command);
        analysis.is_read = Self::is_read_command(command);
        analysis.is_list = Self::is_list_command(command);
        analysis.is_silent = Self::is_silent_command(command);
        analysis.is_destructive = Self::is_destructive_command(command);

        // 安全检查
        analysis.accesses_sensitive_path = Self::accesses_sensitive_paths(command);
        analysis.is_dangerous = Self::is_dangerous_command(command);
        if analysis.is_dangerous {
            analysis.danger_reason = Self::get_danger_reason(command);
        }

        // 沙箱判断
        analysis.can_sandbox = Self::can_run_in_sandbox(command);

        analysis
    }

    /// 提取命令首词
    fn extract_command_prefix(command: &str) -> String {
        // 处理管道、重定向等
        let first_part = command.trim_start();

        // 提取第一个命令
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;
        let mut quote_char = None;
        let mut escaped = false;

        for c in first_part.chars() {
            if escaped {
                current.push(c);
                escaped = false;
                continue;
            }

            if c == '\\' {
                current.push(c);
                escaped = true;
                continue;
            }

            if c == '"' || c == '\'' {
                if !in_quote {
                    in_quote = true;
                    quote_char = Some(c);
                } else if Some(c) == quote_char {
                    in_quote = false;
                    quote_char = None;
                }
                current.push(c);
                continue;
            }

            if !in_quote && c.is_whitespace() {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current = String::new();
                }
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        // 处理命令替换和管道
        parts.first().map(|s| s.clone()).unwrap_or_else(|| {
            // 检查是否以管道或重定向开始
            if first_part.starts_with('|') {
                "pipeline".to_string()
            } else if first_part.starts_with('>') || first_part.starts_with('<') {
                "redirect".to_string()
            } else {
                String::new()
            }
        })
    }

    /// 判断是否为搜索命令
    fn is_search_command(command: &str) -> bool {
        let _search_commands: HashSet<&str> = [
            "grep", "rg", "ag", "find", "locate", "which", "whereis", "type",
        ]
        .iter()
        .cloned()
        .collect();

        command.starts_with("grep")
            || command.starts_with("rg ")
            || command.starts_with("ag ")
            || command.starts_with("find ")
            || command.starts_with("locate ")
            || command.starts_with("which ")
            || command.starts_with("whereis ")
            || command.starts_with("type ")
    }

    /// 判断是否为读取命令
    fn is_read_command(command: &str) -> bool {
        let _read_commands: HashSet<&str> = [
            "cat",
            "head",
            "tail",
            "less",
            "more",
            "wc",
            "stat",
            "jq",
            "awk",
            "sed",
            "file",
            "md5sum",
            "sha1sum",
            "sha256sum",
            "xxd",
            "od",
        ]
        .iter()
        .cloned()
        .collect();

        command.starts_with("cat ")
            || command.starts_with("head ")
            || command.starts_with("tail ")
            || command.starts_with("less ")
            || command.starts_with("more ")
            || command.starts_with("wc ")
            || command.starts_with("stat ")
            || command.starts_with("jq ")
            || command.starts_with("awk ")
            || command.starts_with("sed ")
            || command.starts_with("file ")
            || command.starts_with("md5sum ")
            || command.starts_with("sha1sum ")
            || command.starts_with("sha256sum ")
            || command.starts_with("xxd ")
            || command.starts_with("od ")
    }

    /// 判断是否为列表命令
    fn is_list_command(command: &str) -> bool {
        command.starts_with("ls ")
            || command.starts_with("tree ")
            || command.starts_with("du ")
            || command.starts_with("df ")
            || command == "ls"
            || command == "tree"
            || command.starts_with("ls -")
            || command.starts_with("tree ")
    }

    /// 判断是否为静默命令
    fn is_silent_command(command: &str) -> bool {
        command.starts_with("cp ")
            || command.starts_with("mv ")
            || command.starts_with("mkdir ")
            || command.starts_with("rm ")
            || command.starts_with("chmod ")
            || command.starts_with("chown ")
            || command.starts_with("touch ")
    }

    /// 判断是否为破坏性操作
    fn is_destructive_command(command: &str) -> bool {
        // rm 命令
        if command.starts_with("rm ") {
            return true;
        }

        // 删除目录
        if command.contains("rm -rf") || command.contains("rm -fr") {
            return true;
        }

        // 格式化命令
        if command.starts_with("mkfs") || command.starts_with("dd if=") {
            return true;
        }

        // 覆盖写入
        if command.contains("> /etc/") || command.contains("> /var/") || command.contains("> /usr/")
        {
            return true;
        }

        false
    }

    /// 判断是否访问敏感路径
    fn accesses_sensitive_paths(command: &str) -> bool {
        let sensitive_paths = [
            "/etc/",
            "/etc/passwd",
            "/etc/shadow",
            "/etc/sudoers",
            "/var/",
            "/usr/",
            "/bin/",
            "/sbin/",
            "/lib/",
            "/root/",
            "/boot/",
            "/proc/",
            "/sys/",
            "/dev/",
            "/.env",
            "/.git/",
            "/.ssh/",
            "/.npmrc",
            "/.pypirc",
            "id_rsa",
            ".pem",
            ".key",
            "credentials",
        ];

        sensitive_paths.iter().any(|path| command.contains(path))
    }

    /// 判断是否为危险命令
    fn is_dangerous_command(command: &str) -> bool {
        // 危险模式检测
        if Self::is_dangerous_pattern(command) {
            return true;
        }

        // 访问敏感路径
        if Self::accesses_sensitive_paths(command) {
            return true;
        }

        // 系统命令
        if Self::is_system_admin_command(command) {
            return true;
        }

        false
    }

    /// 检查危险模式
    fn is_dangerous_pattern(command: &str) -> bool {
        let dangerous_patterns = [
            ("rm -rf /", "Attempt to remove root filesystem"),
            ("rm -rf /*", "Attempt to remove all files"),
            ("rm -rf ~", "Attempt to remove home directory"),
            ("rm -rf $HOME", "Attempt to remove home directory"),
            ("mkfs", "Filesystem creation command"),
            ("> /dev/sd", "Direct disk write"),
            ("> /dev/mem", "Memory write attempt"),
            (":(){ :|:& };:", "Fork bomb"),
            (":(){:|:&};:", "Fork bomb"),
            ("chmod -R 777 /", "Dangerous permission change"),
            ("chmod -R 000 /", "Dangerous permission lock"),
            ("chown -R", "Recursive ownership change"),
            ("curl * | sudo", "Pipe curl to sudo"),
            ("wget * | sudo", "Pipe wget to sudo"),
            ("curl * | bash", "Pipe curl to bash"),
            ("wget * | bash", "Pipe wget to bash"),
            ("sudo rm", "Sudo with rm"),
            ("npm publish", "NPM publish operation"),
            ("git push --force", "Force push to git"),
            ("docker system prune", "Docker cleanup"),
        ];

        dangerous_patterns
            .iter()
            .any(|(pattern, _)| command.contains(pattern))
    }

    /// 检查是否为系统管理命令
    fn is_system_admin_command(command: &str) -> bool {
        let sysadmin_commands = [
            "systemctl",
            "service",
            "journalctl",
            "iptables",
            "ufw",
            "firewall-cmd",
            "nftables",
            "fdisk",
            "parted",
            "mkfs",
            "mount",
            "umount",
            "useradd",
            "userdel",
            "usermod",
            "passwd",
            "visudo",
            "vi /etc/sudoers",
            "chroot",
            "grub",
            "update-grub",
            "shutdown",
            "reboot",
            "poweroff",
            "init ",
            "telinit",
            "modprobe",
            "insmod",
            "rmmod",
        ];

        sysadmin_commands
            .iter()
            .any(|cmd| command.starts_with(cmd) || command.contains(format!(" {}", cmd).as_str()))
    }

    /// 获取危险原因
    fn get_danger_reason(command: &str) -> Option<String> {
        let dangerous_patterns = [
            (
                "rm -rf /",
                "Attempt to remove root filesystem - extremely dangerous",
            ),
            (
                "rm -rf /*",
                "Attempt to remove all files from root - will destroy system",
            ),
            (
                "rm -rf ~",
                "Attempt to remove home directory - will lose all user data",
            ),
            (
                "mkfs",
                "Filesystem creation will destroy all data on the device",
            ),
            ("> /dev/sd", "Direct disk write can corrupt storage devices"),
            ("Fork bomb", "Fork bomb will exhaust system resources"),
            (
                "chmod -R 777 /",
                "Opening all permissions to everyone - major security risk",
            ),
            (
                "chmod -R 000 /",
                "Removing all permissions - will lock out all access",
            ),
            (
                "Pipe curl/wget to sudo/bash",
                "Running remote code with elevated privileges",
            ),
            ("sudo rm", "Using rm with sudo privileges"),
            ("npm publish", "Publishing package to registry"),
            (
                "git push --force",
                "Force push can overwrite shared history",
            ),
        ];

        for (pattern, reason) in dangerous_patterns.iter() {
            if command.contains(pattern) {
                return Some(reason.to_string());
            }
        }

        // 检查敏感路径
        if Self::accesses_sensitive_paths(command) {
            return Some("Accessing system or sensitive directories".to_string());
        }

        // 检查系统命令
        if Self::is_system_admin_command(command) {
            return Some("System administration command requires elevated privileges".to_string());
        }

        None
    }

    /// 判断命令是否可以在沙箱中执行
    fn can_run_in_sandbox(command: &str) -> bool {
        // 危险命令不能在沙箱中执行
        if Self::is_dangerous_command(command) {
            return false;
        }

        // 需要网络访问的命令可以在沙箱中执行（取决于沙箱配置）
        if command.starts_with("curl ")
            || command.starts_with("wget ")
            || command.starts_with("ping ")
        {
            return true;
        }

        // 只读命令可以在沙箱中执行
        if Self::is_read_command(command) || Self::is_search_command(command) {
            return true;
        }

        // 其他命令默认可以在沙箱中执行
        true
    }
}

/// 权限规则匹配器（用于 hook 条件）
pub type PermissionMatcher = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// 为 Bash 工具准备权限匹配器
///
/// 匹配器可以根据命令内容进行复杂的模式匹配
pub fn prepare_bash_permission_matcher(command: &str) -> Option<PermissionMatcher> {
    // 提取命令前缀
    let prefix = command.split_whitespace().next()?.to_string();

    // 创建匹配器闭包
    let matcher = move |input_command: &str| -> bool { input_command.starts_with(&prefix) };

    Some(Box::new(matcher))
}

/// 检查命令是否在允许的白名单中
pub fn is_command_allowed(command: &str, whitelist: &[&str]) -> bool {
    // 精确匹配
    if whitelist.contains(&command) {
        return true;
    }

    // 检查白名单中的模式
    for pattern in whitelist {
        if pattern.ends_with(":*") {
            // 前缀匹配
            let prefix = &pattern[..pattern.len() - 2];
            if command.starts_with(prefix) {
                return true;
            }
        } else if pattern.contains('*') {
            // 通配符匹配（简化实现）
            if simple_glob_match(pattern, command) {
                return true;
            }
        }
    }

    false
}

/// 简单的 glob 匹配实现
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    // 将 * 转换为正则表达式 .* 并匹配
    let regex_pattern = pattern.replace('*', ".*");
    let _re_pattern = format!("^{}$", regex_pattern);

    // 简化的正则匹配（避免依赖 regex crate）
    if regex_pattern == ".*" {
        return true;
    }

    text.starts_with(&regex_pattern.replace(".*", ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_search_command() {
        let result = BashSemanticAnalyzer::analyze_command("grep -r 'foo' src/");
        assert!(result.is_search);
        assert!(!result.is_read);
        assert!(!result.is_destructive);
        assert!(!result.is_dangerous);
    }

    #[test]
    fn test_analyze_read_command() {
        let result = BashSemanticAnalyzer::analyze_command("cat src/main.rs");
        assert!(!result.is_search);
        assert!(result.is_read);
        assert!(!result.is_destructive);
        assert!(!result.is_dangerous);
    }

    #[test]
    fn test_analyze_list_command() {
        let result = BashSemanticAnalyzer::analyze_command("ls -la src/");
        assert!(result.is_list);
        assert!(!result.is_search);
        assert!(!result.is_read);
    }

    #[test]
    fn test_analyze_destructive_command() {
        let result = BashSemanticAnalyzer::analyze_command("rm -rf /tmp/test");
        assert!(result.is_destructive);
        assert!(!result.is_dangerous); // 不是绝对危险，只是破坏性
    }

    #[test]
    fn test_analyze_dangerous_command() {
        let result = BashSemanticAnalyzer::analyze_command("rm -rf /");
        assert!(result.is_dangerous);
        assert!(result.danger_reason.is_some());
        assert!(result.danger_reason.unwrap().contains("root filesystem"));
    }

    #[test]
    fn test_analyze_sensitive_path() {
        let result = BashSemanticAnalyzer::analyze_command("cat /etc/passwd");
        assert!(result.accesses_sensitive_path);
        assert!(result.is_dangerous);
    }

    #[test]
    fn test_sandbox_capability() {
        // 只读命令可以沙箱
        let result = BashSemanticAnalyzer::analyze_command("cat file.txt");
        assert!(result.can_sandbox);

        // 危险命令不能沙箱
        let result = BashSemanticAnalyzer::analyze_command("rm -rf /");
        assert!(!result.can_sandbox);
    }

    #[test]
    fn test_command_prefix_extraction() {
        assert_eq!(
            BashSemanticAnalyzer::extract_command_prefix("git status"),
            "git"
        );
        assert_eq!(
            BashSemanticAnalyzer::extract_command_prefix("npm run build"),
            "npm"
        );
        assert_eq!(
            BashSemanticAnalyzer::extract_command_prefix("  ls -la  "),
            "ls"
        );
    }

    #[test]
    fn test_is_command_allowed() {
        let whitelist = ["git:*, npm test, npm run:*"];

        assert!(is_command_allowed("git status", &whitelist));
        assert!(is_command_allowed("git commit -m test", &whitelist));
        assert!(is_command_allowed("npm test", &whitelist));
        assert!(is_command_allowed("npm run build", &whitelist));
        assert!(!is_command_allowed("npm publish", &whitelist));
        assert!(!is_command_allowed("yarn test", &whitelist));
    }
}
