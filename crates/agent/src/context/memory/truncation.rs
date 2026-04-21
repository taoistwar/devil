//! MEMORY.md Truncation
//!
//! Handles truncation of MEMORY.md content to prevent token overflow.
//! - MAX_ENTRYPOINT_LINES: 200 lines
//! - MAX_ENTRYPOINT_BYTES: 25KB

use std::fmt;

pub const MAX_ENTRYPOINT_LINES: usize = 200;
pub const MAX_ENTRYPOINT_BYTES: usize = 25_000;

/// Result of truncating MEMORY.md content
#[derive(Debug, Clone)]
pub struct EntrypointTruncation {
    pub content: String,
    pub line_count: usize,
    pub byte_count: usize,
    pub was_line_truncated: bool,
    pub was_byte_truncated: bool,
}

impl EntrypointTruncation {
    /// Get the truncation reason for warning message
    pub fn truncation_reason(&self) -> String {
        match (&self.was_line_truncated, &self.was_byte_truncated) {
            (true, false) => format!("{} lines (limit: {})", self.line_count, MAX_ENTRYPOINT_LINES),
            (false, true) => format!(
                "{} (limit: {}) — index entries are too long",
                format_size(self.byte_count),
                format_size(MAX_ENTRYPOINT_BYTES)
            ),
            (true, true) => format!(
                "{} lines and {}",
                self.line_count,
                format_size(self.byte_count)
            ),
            (false, false) => String::new(),
        }
    }

    /// Check if any truncation happened
    pub fn was_truncated(&self) -> bool {
        self.was_line_truncated || self.was_byte_truncated
    }
}

/// Format byte size for display
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Truncate MEMORY.md content to line and byte limits
///
/// Line truncation happens first (natural boundary), then byte truncation
/// at the last newline before the limit.
pub fn truncate_entrypoint(raw: &str) -> EntrypointTruncation {
    let trimmed = raw.trim();
    let content_lines = trimmed.split('\n').collect::<Vec<_>>();
    let line_count = content_lines.len();
    let byte_count = trimmed.len();

    let was_line_truncated = line_count > MAX_ENTRYPOINT_LINES;
    let was_byte_truncated = byte_count > MAX_ENTRYPOINT_BYTES;

    if !was_line_truncated && !was_byte_truncated {
        return EntrypointTruncation {
            content: trimmed.to_string(),
            line_count,
            byte_count,
            was_line_truncated: false,
            was_byte_truncated: false,
        };
    }

    let truncated = if was_line_truncated {
        content_lines[..MAX_ENTRYPOINT_LINES].join("\n")
    } else {
        trimmed.to_string()
    };

    let truncated_len = truncated.len();
    let final_content = if truncated_len > MAX_ENTRYPOINT_BYTES {
        let cut_at = truncated.rfind('\n').map(|pos| pos.min(MAX_ENTRYPOINT_BYTES)).unwrap_or(MAX_ENTRYPOINT_BYTES);
        truncated[..cut_at].to_string()
    } else {
        truncated
    };

    EntrypointTruncation {
        content: final_content,
        line_count,
        byte_count,
        was_line_truncated,
        was_byte_truncated,
    }
}

impl fmt::Display for EntrypointTruncation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Content length: {} lines, {} bytes", self.line_count, self.byte_count)?;
        if self.was_truncated() {
            writeln!(f, "WARNING: MEMORY.md was truncated. Reason: {}", self.truncation_reason())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_truncation_needed() {
        let content = "line1\nline2\nline3";
        let result = truncate_entrypoint(content);
        assert!(!result.was_truncated());
        assert_eq!(result.content, content);
    }

    #[test]
    fn test_line_truncation() {
        let lines: Vec<String> = (0..250).map(|i| format!("line {}", i)).collect();
        let content = lines.join("\n");
        let result = truncate_entrypoint(&content);
        assert!(result.was_line_truncated);
        assert_eq!(result.line_count, 250);
    }

    #[test]
    fn test_byte_truncation() {
        let content = "x".repeat(30_000);
        let result = truncate_entrypoint(&content);
        assert!(result.was_byte_truncated);
        assert!(result.content.len() <= MAX_ENTRYPOINT_BYTES);
    }

    #[test]
    fn test_truncation_reason() {
        let lines: Vec<String> = (0..250).map(|i| format!("line {}", i)).collect();
        let content = lines.join("\n");
        let result = truncate_entrypoint(&content);
        assert!(result.truncation_reason().contains("lines"));
    }
}
