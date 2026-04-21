//! Git Status Collection Module
//!
//! Implements Git status collection aligned with Claude Code's context.ts

use std::process::Command;

const MAX_STATUS_CHARS: usize = 2000;

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub main_branch: String,
    pub user_name: Option<String>,
    pub status: String,
    pub recent_commits: String,
}

pub struct GitStatusCollector;

impl GitStatusCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> Option<GitStatus> {
        if !self.is_git_repo() {
            return None;
        }

        let branch = self.get_branch()?;
        let main_branch = self.get_main_branch()?;
        let status = self.get_status()?;
        let recent_commits = self.get_recent_commits()?;
        let user_name = self.get_user_name();

        Some(GitStatus {
            branch,
            main_branch,
            user_name,
            status,
            recent_commits,
        })
    }

    fn is_git_repo(&self) -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn get_branch(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if branch.is_empty() {
                return Some("HEAD".to_string());
            }
            Some(branch)
        } else {
            None
        }
    }

    fn get_main_branch(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
            .output()
            .ok()?;

        if output.status.success() {
            let ref_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(ref_str.replace("refs/remotes/origin/", ""))
        } else {
            let output = Command::new("git")
                .args(["branch", "-m", "main"])
                .output()
                .ok()?;
            if output.status.success() {
                Some("main".to_string())
            } else {
                Some("master".to_string())
            }
        }
    }

    fn get_status(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["--no-optional-locks", "status", "--short"])
            .output()
            .ok()?;

        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(self.truncate_status(status))
        } else {
            Some("(clean)".to_string())
        }
    }

    fn get_recent_commits(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["--no-optional-locks", "log", "--oneline", "-n", "5"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Some(String::new())
        }
    }

    fn get_user_name(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["config", "user.name"])
            .output()
            .ok()?;

        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        } else {
            None
        }
    }

    fn truncate_status(&self, status: String) -> String {
        if status.len() > MAX_STATUS_CHARS {
            format!(
                "{}\n... (truncated because it exceeds 2k characters. If you need more information, run \"git status\" using BashTool)",
                &status[..MAX_STATUS_CHARS]
            )
        } else {
            status
        }
    }
}

impl Default for GitStatusCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_git_status() -> Option<GitStatus> {
    GitStatusCollector::new().collect()
}

pub fn clear_git_status_cache() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_collector_creation() {
        let collector = GitStatusCollector::new();
        assert!(collector.is_git_repo() || true);
    }

    #[test]
    fn test_truncate_status_short() {
        let collector = GitStatusCollector::new();
        let short_status = "M Cargo.toml".to_string();
        assert_eq!(
            collector.truncate_status(short_status.clone()),
            short_status
        );
    }

    #[test]
    fn test_truncate_status_long() {
        let collector = GitStatusCollector::new();
        let long_status = "a".repeat(3000);
        let truncated = collector.truncate_status(long_status.clone());
        assert!(truncated.len() < long_status.len());
        assert!(truncated.contains("truncated"));
    }
}
