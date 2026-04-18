use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverSkillsInput {
    pub path: Option<String>,
    pub reload: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverSkillsOutput {
    pub skills: Vec<SkillInfo>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub path: String,
}

pub struct DiscoverSkillsTool;

impl Default for DiscoverSkillsTool {
    fn default() -> Self {
        Self
    }
}

impl DiscoverSkillsTool {
    fn get_skills_directory() -> PathBuf {
        PathBuf::from("/root/.claude/skills")
    }

    fn discover_skills_from_path(path: &PathBuf) -> Vec<SkillInfo> {
        let mut skills = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let skill_name = entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let description = if let Ok(skill_md) = std::fs::read_to_string(entry_path.join("SKILL.md")) {
                        skill_md.lines().next().unwrap_or("").to_string()
                    } else {
                        String::new()
                    };

                    skills.push(SkillInfo {
                        name: skill_name,
                        description,
                        path: entry_path.to_string_lossy().to_string(),
                    });
                }
            }
        }

        skills
    }
}

#[async_trait]
impl Tool for DiscoverSkillsTool {
    type Input = DiscoverSkillsInput;
    type Output = DiscoverSkillsOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "discover_skills"
    }

    fn aliases(&self) -> &[&str] {
        &["skills_discover", "list_skills"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to search for skills"
                },
                "reload": {
                    "type": "boolean",
                    "description": "Force reload from disk"
                }
            }
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let skills_path = input
            .path
            .map(PathBuf::from)
            .unwrap_or_else(Self::get_skills_directory);

        let skills = Self::discover_skills_from_path(&skills_path);
        let count = skills.len();

        let output = DiscoverSkillsOutput { skills, count };

        Ok(ToolResult::success("discover_skills-1", output))
    }
}
