use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    #[serde(rename = "cell_type")]
    pub cell_type: CellType,
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub source: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellType {
    Code,
    Markdown,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookDocument {
    #[serde(rename = "nbformat_minor")]
    pub nbformat_minor: u32,
    #[serde(rename = "nbformat_major")]
    pub nbformat_major: u32,
    pub metadata: serde_json::Value,
    pub cells: Vec<NotebookCell>,
}

impl Default for NotebookDocument {
    fn default() -> Self {
        Self {
            nbformat_minor: 5,
            nbformat_major: 4,
            metadata: serde_json::json!({}),
            cells: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellUpdate {
    pub index: usize,
    #[serde(rename = "cell_type")]
    pub cell_type: Option<CellType>,
    pub source: Option<Vec<String>>,
    pub outputs: Option<Vec<serde_json::Value>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookEditInput {
    pub path: String,
    pub operation: NotebookOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum NotebookOperation {
    #[serde(rename = "insert")]
    Insert {
        index: usize,
        cell_type: CellType,
        source: Vec<String>,
    },
    #[serde(rename = "delete")]
    Delete { index: usize },
    #[serde(rename = "update")]
    Update(CellUpdate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookEditOutput {
    pub success: bool,
    pub cells_modified: usize,
}
