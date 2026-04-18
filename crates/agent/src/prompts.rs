//! 提示词模块
//!
//! 提供 Agent 使用的各类系统提示词和提示模板

/// 代码库探索提示词
pub mod exploration {
    /// 生成代码库分析的系统提示词
    pub fn codebase_analysis_system_prompt() -> String {
        r#"You are a codebase analysis expert. Your task is to explore and understand the structure of the provided codebase.

## Your Capabilities
- Use Glob tool to find files matching patterns
- Use Grep tool to search for specific patterns, imports, or function definitions
- Use Read tool to examine file contents
- Combine multiple tools to build a comprehensive understanding

## Analysis Framework
When analyzing a codebase, follow this framework:

### 1. Project Structure
- Identify the main entry points (main.rs, index.js, etc.)
- Determine the project type (Web app, CLI tool, library, etc.)
- Map the high-level directory structure

### 2. Technology Stack
- Identify programming languages used
- Find dependency configuration files (Cargo.toml, package.json, go.mod, etc.)
- Identify frameworks and major libraries

### 3. Key Components
- Find core business logic files
- Identify data models and structures
- Locate configuration files

### 4. Dependencies
- Map external dependencies
- Understand how components are connected

## Output Format
Provide your analysis in this format:

```markdown
# Codebase Analysis

## Project Overview
[Brief description of what this project does]

## Structure
[Directory structure with descriptions]

## Technology Stack
- Language: [primary language]
- Framework: [main framework, if any]
- Key Dependencies: [list important libraries]

## Key Files
- [file path]: [what it does]
- ...

## Component Relationships
[How main components connect]
```
"#
        .to_string()
    }

    /// 生成探索任务的提示词
    pub fn exploration_task_prompt(task: &str) -> String {
        format!(
            r#"Your task is to: {}

Please explore the codebase to gather information needed for this task.
Use the available tools (Glob, Grep, Read) to investigate.

Return a summary of your findings including:
1. What you discovered
2. Key files or patterns relevant to the task
3. Any recommendations for next steps"#,
            task
        )
    }
}

/// 文件操作提示词
pub mod file_operations {
    /// 生成文件修改前的确认提示
    pub fn file_modification_confirmation(
        file_path: &str,
        operation: &str,
        details: &str,
    ) -> String {
        format!(
            r#"File Operation Confirmation Required

File: {}
Operation: {}
Details: {}

Do you want to proceed with this operation? (yes/no)"#,
            file_path, operation, details
        )
    }

    /// 生成写入操作的安全警告
    pub fn write_operation_warning(file_path: &str, has_backup: bool) -> String {
        let backup_note = if has_backup {
            "A backup of the original file will be created before making changes."
        } else {
            "Warning: No backup will be created for this operation."
        };

        format!(
            r#"⚠️ Write Operation Warning

File: {}
{}

Do you want to proceed?"#,
            file_path, backup_note
        )
    }

    /// 生成破坏性操作确认
    pub fn destructive_operation_warning(operation: &str, target: &str) -> String {
        format!(
            r#"⚠️ Destructive Operation Warning

This operation will {}: {}

This action cannot be easily undone. Please confirm you want to proceed.

Type 'yes' to confirm: "#,
            operation, target
        )
    }
}

/// 错误恢复提示词
pub mod error_recovery {
    /// 生成错误上下文摘要
    pub fn error_context_summary(error: &str, context: &[&str]) -> String {
        let context_str = context.join("\n- ");
        format!(
            r#"Error Encountered

Error: {}

Context:
- {}

## Recovery Options
1. Try a different approach
2. Gather more information with exploration tools
3. Ask for user guidance
4. Abort the current task"#,
            error,
            if context_str.is_empty() {
                "No additional context".to_string()
            } else {
                context_str
            }
        )
    }

    /// 生成重试建议
    pub fn retry_suggestion(attempt: usize, max_attempts: usize) -> String {
        format!(
            "Attempt {}/{} failed. What would you like to do next?",
            attempt, max_attempts
        )
    }
}

/// 子代理任务提示词
pub mod subagent {
    /// 生成子代理系统提示词
    pub fn subagent_system_prompt(role: &str, task: &str) -> String {
        format!(
            r#"You are a {} subagent.

Your Task: {}

## Guidelines
- Focus only on your assigned task
- Report findings clearly
- Do not attempt tasks outside your scope
- Ask for clarification if instructions are unclear

## Output
Provide your results in a structured format suitable for the parent agent to consume."#,
            role, task
        )
    }

    /// 生成分支任务提示
    pub fn branch_task_prompt(task_description: &str) -> String {
        format!(
            r#"You are assigned a subtask as part of a larger effort.

Task: {}

Work independently to complete this subtask. When done, summarize:
1. What you found/accomplished
2. Any decisions made
3. Recommendations for other subtasks"#,
            task_description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codebase_analysis_prompt() {
        let prompt = exploration::codebase_analysis_system_prompt();
        assert!(prompt.contains("Glob"));
        assert!(prompt.contains("Grep"));
        assert!(prompt.contains("Read"));
        assert!(prompt.contains("Analysis Framework"));
    }

    #[test]
    fn test_confirmation_prompt() {
        let prompt = file_operations::file_modification_confirmation(
            "/path/to/file.txt",
            "write",
            "Adding new content",
        );
        assert!(prompt.contains("/path/to/file.txt"));
        assert!(prompt.contains("write"));
        assert!(prompt.contains("yes/no"));
    }
}
