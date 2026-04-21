use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::Path;

use crate::tools::file_tools::notebook_types::{
    CellUpdate, NotebookDocument, NotebookEditInput, NotebookEditOutput, NotebookOperation,
};
use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

pub struct NotebookEditTool;

impl Default for NotebookEditTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for NotebookEditTool {
    type Input = NotebookEditInput;
    type Output = NotebookEditOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "notebook_edit"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the .ipynb file"
                },
                "operation": {
                    "type": "object",
                    "description": "The operation to perform",
                    "oneOf": [
                        {
                            "type": "object",
                            "properties": {
                                "type": { "const": "insert" },
                                "index": { "type": "integer" },
                                "cell_type": { "type": "string", "enum": ["code", "markdown", "raw"] },
                                "source": { "type": "array", "items": { "type": "string" } }
                            },
                            "required": ["type", "index", "cell_type", "source"]
                        },
                        {
                            "type": "object",
                            "properties": {
                                "type": { "const": "delete" },
                                "index": { "type": "integer" }
                            },
                            "required": ["type", "index"]
                        },
                        {
                            "type": "object",
                            "properties": {
                                "type": { "const": "update" },
                                "index": { "type": "integer" },
                                "cell_type": { "type": "string" },
                                "source": { "type": "array", "items": { "type": "string" } },
                                "outputs": { "type": "array" },
                                "metadata": { "type": "object" }
                            },
                            "required": ["type", "index"]
                        }
                    ]
                }
            },
            "required": ["path", "operation"]
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
        let path = Path::new(&input.path);

        let mut notebook: NotebookDocument = if path.exists() {
            let content = fs::read_to_string(path)?;
            serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Invalid notebook format: {}", e))?
        } else {
            NotebookDocument::default()
        };

        let cells_modified = match input.operation {
            NotebookOperation::Insert {
                index,
                cell_type,
                source,
            } => {
                let cell = crate::tools::file_tools::notebook_types::NotebookCell {
                    cell_type,
                    metadata: serde_json::json!({}),
                    source,
                };
                let index = index.min(notebook.cells.len());
                notebook.cells.insert(index, cell);
                1
            }
            NotebookOperation::Delete { index } => {
                if index < notebook.cells.len() {
                    notebook.cells.remove(index);
                    1
                } else {
                    0
                }
            }
            NotebookOperation::Update(update) => apply_cell_update(&mut notebook, update)?,
        };

        let json = serde_json::to_string_pretty(&notebook)
            .map_err(|e| anyhow::anyhow!("Failed to serialize notebook: {}", e))?;

        fs::write(path, json)?;

        let output = NotebookEditOutput {
            success: true,
            cells_modified,
        };

        Ok(ToolResult::success("notebook_edit-1", output))
    }
}

fn apply_cell_update(notebook: &mut NotebookDocument, update: CellUpdate) -> anyhow::Result<usize> {
    if update.index >= notebook.cells.len() {
        anyhow::bail!("Cell index out of bounds");
    }

    let cell = &mut notebook.cells[update.index];

    if let Some(cell_type) = update.cell_type {
        cell.cell_type = cell_type;
    }

    if let Some(source) = update.source {
        cell.source = source;
    }

    if let Some(metadata) = update.metadata {
        cell.metadata = metadata;
    }

    if let Some(outputs) = update.outputs {
        if let serde_json::Value::Object(ref mut m) = cell.metadata {
            m.insert("outputs".to_string(), serde_json::json!(outputs));
        }
    }

    Ok(1)
}
