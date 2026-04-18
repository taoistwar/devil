//! 使用真实 Anthropic API 的 QueryDeps 实现

use anyhow::{Context, Result};
use futures::future::FutureExt;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::cost_tracking::{update_usage, TokenUsage, UsageDelta};
use crate::query_engine::{ContentBlock, Message, QueryDeps, StreamEvent};
use crate::streaming_tool_executor::{ToolResult, TrackedTool};
use devil_mcp::{MappedTool, McpConnectionManager, PermissionChecker, ToolDiscoverer};
use providers::{
    AnthropicClient, ChatMessage as AnthropicChatMessage, ContentBlock as AnthropicContentBlock,
    ContentBlockStart as AnthropicContentBlockStart, ToolDef,
};

fn convert_usage(usage: &providers::Usage) -> TokenUsage {
    TokenUsage {
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        cache_creation_input_tokens: usage.cache_creation_input_tokens,
        cache_read_input_tokens: usage.cache_read_input_tokens,
    }
}

fn convert_content_block(block: &AnthropicContentBlock) -> ContentBlock {
    match block {
        AnthropicContentBlock::Text { text } => ContentBlock::Text { text: text.clone() },
        AnthropicContentBlock::ToolUse { id, name, input } => ContentBlock::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
        },
        AnthropicContentBlock::ToolResult { .. } => ContentBlock::Text {
            text: String::new(),
        },
    }
}

/// Anthropic-backed QueryDeps 实现
pub struct AnthropicQueryDeps {
    /// Anthropic 客户端
    client: Arc<AnthropicClient>,
    /// MCP 连接管理器
    mcp_manager: Arc<McpConnectionManager>,
    /// 权限检查器
    permission_checker: Arc<PermissionChecker>,
    /// 工具发现器
    tool_discoverer: Arc<ToolDiscoverer>,
    /// MCP 工具名称 -> 全局名称映射
    tool_name_map: Arc<RwLock<HashMap<String, String>>>,
    /// System prompt
    system_prompt: Arc<RwLock<String>>,
}

impl AnthropicQueryDeps {
    /// 创建新的 AnthropicQueryDeps
    pub fn new(
        api_key: String,
        model: Option<String>,
        mcp_manager: Arc<McpConnectionManager>,
        permission_checker: Arc<PermissionChecker>,
        tool_discoverer: Arc<ToolDiscoverer>,
    ) -> Self {
        let client = Arc::new(AnthropicClient::new(api_key, model));

        Self {
            client,
            mcp_manager,
            permission_checker,
            tool_discoverer,
            tool_name_map: Arc::new(RwLock::new(HashMap::new())),
            system_prompt: Arc::new(RwLock::new(
                "你是一个专业的 AI 编程助手，帮助用户完成软件开发任务。".to_string(),
            )),
        }
    }

    /// 设置 system prompt
    pub async fn set_system_prompt(&self, prompt: String) {
        let mut system = self.system_prompt.write().await;
        *system = prompt;
    }

    /// 初始化 MCP 工具
    pub async fn initialize_mcp_tools(&self) -> Result<Vec<MappedTool>> {
        info!("Initializing MCP tools");

        let mut all_tools = Vec::new();
        let servers = self.mcp_manager.list_servers().await;

        for server_id in servers {
            match self.permission_checker.check_server(&server_id).await {
                devil_mcp::PermissionResult::Allowed => {}
                _ => continue,
            }

            let tools = self.tool_discoverer.get_tools(&server_id).await;

            for tool in tools {
                let global_name = tool.global_name.clone();
                let original_name = tool.original_name.clone();

                self.tool_name_map
                    .write()
                    .await
                    .insert(original_name.clone(), global_name.clone());

                match self.permission_checker.check_tool(&global_name).await {
                    devil_mcp::PermissionResult::Allowed => {
                        self.tool_discoverer
                            .update_authorization(&global_name, true)
                            .await?;
                        all_tools.push(tool);
                    }
                    _ => {}
                }
            }
        }

        info!("Discovered {} MCP tools", all_tools.len());
        Ok(all_tools)
    }

    /// 转换 Message 为 Anthropic format
    async fn convert_messages(&self, messages: &[Message]) -> Vec<AnthropicChatMessage> {
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg {
                Message::User { content } => {
                    anthropic_messages.push(AnthropicChatMessage {
                        role: "user".to_string(),
                        content: vec![AnthropicContentBlock::Text {
                            text: content.clone(),
                        }],
                    });
                }
                Message::Assistant { content, .. } => {
                    let anthropic_content: Vec<AnthropicContentBlock> = content
                        .iter()
                        .map(|c| convert_content_block_anthropic(c))
                        .collect();

                    anthropic_messages.push(AnthropicChatMessage {
                        role: "assistant".to_string(),
                        content: anthropic_content,
                    });
                }
                Message::ToolResult {
                    tool_use_id,
                    content,
                    is_error,
                } => {
                    anthropic_messages.push(AnthropicChatMessage {
                        role: "user".to_string(),
                        content: vec![AnthropicContentBlock::ToolResult {
                            tool_use_id: tool_use_id.clone(),
                            content: content.clone(),
                            is_error: if *is_error { Some(true) } else { None },
                        }],
                    });
                }
            }
        }

        anthropic_messages
    }

    /// 转换工具定义为 Anthropic format
    async fn convert_tools(&self) -> Vec<ToolDef> {
        let tools = self.tool_discoverer.get_authorized_tools().await;
        let mut tool_defs = Vec::new();

        for tool in tools {
            tool_defs.push(ToolDef {
                name: tool.original_name.clone(),
                description: tool.description.clone(),
                input_schema: tool.input_schema.clone(),
            });
        }

        tool_defs
    }

    /// 执行 MCP 工具
    pub async fn execute_mcp_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String> {
        let tool_map = self.tool_name_map.read().await;
        let global_name = tool_map
            .get(tool_name)
            .with_context(|| format!("Tool not found: {}", tool_name))?;

        match self.permission_checker.check_tool(global_name).await {
            devil_mcp::PermissionResult::Allowed => {}
            devil_mcp::PermissionResult::Denied(reason) => {
                anyhow::bail!("Tool {} denied: {}", global_name, reason);
            }
            devil_mcp::PermissionResult::NeedsConfirmation => {
                anyhow::bail!("Tool {} needs confirmation", global_name);
            }
        }

        let parts: Vec<&str> = global_name.split("__").collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid tool name format: {}", global_name);
        }
        let server_id = parts[1];

        // TODO: 实际调用 MCP Bridge
        // let bridge = self.mcp_manager.get_bridge(server_id).await?;
        // let result = bridge.call_tool(parts[2], arguments).await?;

        info!("Executing MCP tool: {} on server {}", tool_name, server_id);

        let result = serde_json::json!({
            "status": "success",
            "tool": tool_name,
            "server": server_id,
            "arguments": arguments,
        });

        Ok(result.to_string())
    }
}

fn convert_content_block_anthropic(block: &ContentBlock) -> AnthropicContentBlock {
    match block {
        ContentBlock::Text { text } => AnthropicContentBlock::Text { text: text.clone() },
        ContentBlock::ToolUse { id, name, input } => AnthropicContentBlock::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
        },
    }
}

impl QueryDeps for AnthropicQueryDeps {
    fn call_model(
        &self,
        messages: &[Message],
        _stream: bool,
    ) -> futures::stream::BoxStream<'static, Result<StreamEvent>> {
        let client = self.client.clone();
        let system_prompt = self.system_prompt.clone();
        let messages = messages.to_vec();

        // 转换为 Anthropic 格式
        let anthropic_messages = messages
            .iter()
            .map(|msg| match msg {
                Message::User { content } => AnthropicChatMessage {
                    role: "user".to_string(),
                    content: vec![AnthropicContentBlock::Text {
                        text: content.clone(),
                    }],
                },
                Message::Assistant { content, .. } => {
                    let anthropic_content: Vec<AnthropicContentBlock> = content
                        .iter()
                        .map(|c| convert_content_block_anthropic(c))
                        .collect();
                    AnthropicChatMessage {
                        role: "assistant".to_string(),
                        content: anthropic_content,
                    }
                }
                Message::ToolResult {
                    tool_use_id,
                    content,
                    is_error,
                } => AnthropicChatMessage {
                    role: "user".to_string(),
                    content: vec![AnthropicContentBlock::ToolResult {
                        tool_use_id: tool_use_id.clone(),
                        content: content.clone(),
                        is_error: if *is_error { Some(true) } else { None },
                    }],
                },
            })
            .collect();

        use futures::stream;

        // 调用 Anthropic API（流式）
        async move {
            let system = system_prompt.read().await.clone();
            let tools = None; // 工具已经在 messages 中的 tool_use 中体现

            match client
                .chat_stream(anthropic_messages, Some(system), tools)
                .await
            {
                Ok(stream) => stream
                    .map(|result| {
                        result.map(|event| match event {
                            providers::StreamEvent::MessageStart { message } => {
                                StreamEvent::MessageStart { id: message.id }
                            }
                            providers::StreamEvent::ContentBlockStart {
                                index,
                                content_block,
                            } => StreamEvent::ContentBlockDelta {
                                block_type: crate::query_engine::BlockType::Text,
                                delta: crate::query_engine::ContentDelta::TextDelta {
                                    text: match content_block {
                                        AnthropicContentBlockStart::Text { text } => text,
                                        AnthropicContentBlockStart::ToolUse { .. } => String::new(),
                                    },
                                },
                            },
                            providers::StreamEvent::ContentBlockDelta { index, delta } => {
                                match delta {
                                    providers::ContentDelta::TextDelta { text } => {
                                        StreamEvent::ContentBlockDelta {
                                            block_type: crate::query_engine::BlockType::Text,
                                            delta: crate::query_engine::ContentDelta::TextDelta {
                                                text,
                                            },
                                        }
                                    }
                                    providers::ContentDelta::InputJsonDelta { partial_json } => {
                                        StreamEvent::ContentBlockDelta {
                                            block_type: crate::query_engine::BlockType::ToolUse,
                                            delta: crate::query_engine::ContentDelta::InputDelta {
                                                partial_json,
                                            },
                                        }
                                    }
                                }
                            }
                            providers::StreamEvent::MessageDelta { delta, usage } => {
                                let usage_delta = UsageDelta {
                                    input_tokens: Some(usage.input_tokens),
                                    output_tokens: Some(usage.output_tokens),
                                    cache_creation_input_tokens: Some(
                                        usage.cache_creation_input_tokens,
                                    ),
                                    cache_read_input_tokens: Some(usage.cache_read_input_tokens),
                                };
                                StreamEvent::MessageDelta {
                                    usage: usage_delta,
                                    stop_reason: delta.stop_reason.map(|s| match s.as_str() {
                                        "end_turn" => crate::query_engine::StopReason::EndTurn,
                                        "tool_use" => crate::query_engine::StopReason::ToolUse,
                                        "max_tokens" => crate::query_engine::StopReason::MaxTokens,
                                        _ => crate::query_engine::StopReason::EndTurn,
                                    }),
                                }
                            }
                            providers::StreamEvent::ContentBlockStop { index } => {
                                StreamEvent::MessageStop
                            }
                            providers::StreamEvent::MessageStop => StreamEvent::MessageStop,
                            providers::StreamEvent::Ping => StreamEvent::Progress {
                                message: "ping".to_string(),
                            },
                        })
                    })
                    .boxed(),
                Err(e) => {
                    error!("Anthropic API error: {}", e);
                    stream::empty().boxed()
                }
            }
        }
        .flatten_stream()
        .boxed()
    }

    fn execute_tool(
        &self,
        tool: &TrackedTool,
    ) -> futures::future::BoxFuture<'static, Result<String>> {
        let this = self.clone();
        let tool_name = tool.name.clone();
        let arguments = tool.input.clone();

        async move { this.execute_mcp_tool(&tool_name, arguments).await }.boxed()
    }
}

impl Clone for AnthropicQueryDeps {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            mcp_manager: self.mcp_manager.clone(),
            permission_checker: self.permission_checker.clone(),
            tool_discoverer: self.tool_discoverer.clone(),
            tool_name_map: self.tool_name_map.clone(),
            system_prompt: self.system_prompt.clone(),
        }
    }
}
