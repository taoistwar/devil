//! /memory 命令 - 记忆管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use crate::context::memory::{MemoryDir, MemoryEntry, MemoryFrontmatter, MemoryType};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct MemoryCommand;

impl MemoryCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, serde::Serialize)]
pub enum MemoryAction {
    List,
    ListByType,
    Add,
    Delete,
    Help,
}

#[derive(Debug, serde::Serialize)]
pub struct MemoryResponse {
    pub action: MemoryAction,
    pub memories: Vec<MemoryInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct MemoryInfo {
    pub name: String,
    pub description: String,
    pub memory_type: String,
    pub path: String,
}

impl MemoryResponse {
    fn list_memories(memories: Vec<MemoryEntry>) -> Self {
        let infos: Vec<MemoryInfo> = memories
            .into_iter()
            .map(|e| MemoryInfo {
                name: e.frontmatter.name,
                description: e.frontmatter.description,
                memory_type: e.frontmatter.memory_type.to_string(),
                path: e.path.to_string_lossy().to_string(),
            })
            .collect();

        Self {
            action: MemoryAction::List,
            memories: infos,
            message: None,
        }
    }

    fn list_by_type(memories: Vec<MemoryEntry>, memory_type: MemoryType) -> Self {
        let count = memories.len();
        let infos: Vec<MemoryInfo> = memories
            .into_iter()
            .map(|e| MemoryInfo {
                name: e.frontmatter.name,
                description: e.frontmatter.description,
                memory_type: e.frontmatter.memory_type.to_string(),
                path: e.path.to_string_lossy().to_string(),
            })
            .collect();

        Self {
            action: MemoryAction::ListByType,
            memories: infos,
            message: Some(format!("{} 类型记忆共 {} 条", memory_type, count)),
        }
    }

    fn add_memory(name: &str, description: &str, memory_type: MemoryType, path: PathBuf) -> Self {
        Self {
            action: MemoryAction::Add,
            memories: vec![MemoryInfo {
                name: name.to_string(),
                description: description.to_string(),
                memory_type: memory_type.to_string(),
                path: path.to_string_lossy().to_string(),
            }],
            message: Some("记忆已保存".to_string()),
        }
    }

    fn delete_memory(path: &str) -> Self {
        Self {
            action: MemoryAction::Delete,
            memories: vec![],
            message: Some(format!("已删除: {}", path)),
        }
    }

    fn help() -> Self {
        Self {
            action: MemoryAction::Help,
            memories: vec![],
            message: Some(
                "用法: /memory [list|add <type> <name> <description>|delete <path>]\n\
                 示例:\n  /memory list                    # 列出所有记忆\n  /memory list user               # 列出用户类型记忆\n  /memory add user 我的角色 我是后端开发  # 添加用户记忆\n  /memory delete user_test.md     # 删除指定记忆".to_string(),
            ),
        }
    }
}

#[async_trait]
impl SlashCommand for MemoryCommand {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "记忆管理 (list/add/delete)"
    }

    fn usage(&self) -> &str {
        "/memory [list|add <type> <name> <desc>|delete <path>]"
    }

    async fn execute(&self, _ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let memory_dir = MemoryDir::resolve();

        if args.is_empty() || args[0] == "list" {
            match memory_dir.list_memories().await {
                Ok(memories) => {
                    let response = MemoryResponse::list_memories(memories);
                    CommandResult::success_with_data("记忆列表", serde_json::to_value(response).unwrap())
                }
                Err(e) => CommandResult::error(format!("加载记忆失败: {}", e)),
            }
        } else {
            match args[0] {
                "list" if args.len() > 1 => {
                    if let Some(mt) = MemoryType::from_str(args[1]) {
                        match memory_dir.list_memories_by_type(mt).await {
                            Ok(memories) => {
                                let response = MemoryResponse::list_by_type(memories, mt);
                                CommandResult::success_with_data("记忆列表", serde_json::to_value(response).unwrap())
                            }
                            Err(e) => CommandResult::error(format!("加载记忆失败: {}", e)),
                        }
                    } else {
                        CommandResult::error(format!("未知记忆类型: {}", args[1]))
                    }
                }

                "add" if args.len() >= 4 => {
                    if let Some(memory_type) = MemoryType::from_str(args[1]) {
                        let name = args[2];
                        let description = args[3];
                        let path = memory_dir.memory_path(memory_type, name);

                        let entry = MemoryEntry::new(
                            MemoryFrontmatter {
                                name: name.to_string(),
                                description: description.to_string(),
                                memory_type,
                            },
                            String::new(),
                            path.clone(),
                        );

                        match memory_dir.save_memory(&entry).await {
                            Ok(_) => {
                                let response = MemoryResponse::add_memory(name, description, memory_type, path);
                                CommandResult::success_with_data("添加记忆", serde_json::to_value(response).unwrap())
                            }
                            Err(e) => CommandResult::error(format!("保存记忆失败: {}", e)),
                        }
                    } else {
                        CommandResult::error(format!("未知记忆类型: {}", args[1]))
                    }
                }

                "delete" if args.len() > 1 => {
                    let path = PathBuf::from(args[1]);
                    match memory_dir.delete_memory(&path).await {
                        Ok(_) => {
                            let response = MemoryResponse::delete_memory(args[1]);
                            CommandResult::success_with_data("删除记忆", serde_json::to_value(response).unwrap())
                        }
                        Err(e) => CommandResult::error(format!("删除记忆失败: {}", e)),
                    }
                }

                _ => {
                    let response = MemoryResponse::help();
                    CommandResult::success_with_data("记忆管理", serde_json::to_value(response).unwrap())
                }
            }
        }
    }
}
