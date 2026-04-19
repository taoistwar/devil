use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCreateInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCreateOutput {
    pub id: String,
    pub name: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDeleteInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDeleteOutput {
    pub id: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInfo {
    pub id: String,
    pub name: String,
    pub members: Vec<TeamMember>,
}

pub struct TeamStore {
    teams: Arc<RwLock<HashMap<String, TeamInfo>>>,
}

impl Default for TeamStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamStore {
    pub fn new() -> Self {
        Self {
            teams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, name: &str, _description: &str) -> TeamInfo {
        let id = Uuid::new_v4().to_string();
        let team = TeamInfo {
            id: id.clone(),
            name: name.to_string(),
            members: vec![TeamMember {
                id: Uuid::new_v4().to_string(),
                name: "owner".to_string(),
                role: "owner".to_string(),
            }],
        };
        let mut teams = self.teams.write().await;
        teams.insert(id, team.clone());
        team
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut teams = self.teams.write().await;
        teams.remove(id).is_some()
    }

    pub async fn get(&self, id: &str) -> Option<TeamInfo> {
        let teams = self.teams.read().await;
        teams.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<TeamInfo> {
        let teams = self.teams.read().await;
        teams.values().cloned().collect()
    }
}

pub struct TeamCreateTool {
    store: TeamStore,
}

impl TeamCreateTool {
    pub fn new(store: TeamStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for TeamCreateTool {
    type Input = TeamCreateInput;
    type Output = TeamCreateOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "team_create"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Team name"
                },
                "description": {
                    "type": "string",
                    "description": "Team description"
                }
            },
            "required": ["name"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
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
        let team = self.store.create(&input.name, input.description.as_deref().unwrap_or("")).await;

        let output = TeamCreateOutput {
            id: team.id,
            name: team.name,
            success: true,
        };

        Ok(ToolResult::success("team_create-1", output))
    }
}

pub struct TeamDeleteTool {
    store: TeamStore,
}

impl TeamDeleteTool {
    pub fn new(store: TeamStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for TeamDeleteTool {
    type Input = TeamDeleteInput;
    type Output = TeamDeleteOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "team_delete"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Team ID to delete"
                }
            },
            "required": ["id"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
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
        let success = self.store.delete(&input.id).await;

        let output = TeamDeleteOutput {
            id: input.id,
            success,
        };

        Ok(ToolResult::success("team_delete-1", output))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPeersInput {
    pub team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPeersOutput {
    pub peers: Vec<PeerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

pub struct ListPeersTool;

impl Default for ListPeersTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListPeersTool {
    type Input = ListPeersInput;
    type Output = ListPeersOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "list_peers"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "team_id": {
                    "type": "string",
                    "description": "Team ID to list peers from"
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
        _input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = ListPeersOutput {
            peers: Vec::new(),
        };

        Ok(ToolResult::success("list_peers-1", output))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageInput {
    pub peer_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageOutput {
    pub peer_id: String,
    pub success: bool,
}

pub struct SendMessageTool;

impl Default for SendMessageTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SendMessageTool {
    type Input = SendMessageInput;
    type Output = SendMessageOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "send_message"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "peer_id": {
                    "type": "string",
                    "description": "Peer ID to send message to"
                },
                "message": {
                    "type": "string",
                    "description": "Message content"
                }
            },
            "required": ["peer_id", "message"]
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

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = SendMessageOutput {
            peer_id: input.peer_id,
            success: true,
        };

        Ok(ToolResult::success("send_message-1", output))
    }
}
