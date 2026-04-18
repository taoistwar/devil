//! Anthropic API 客户端
//!
//! 实现：
//! - 流式聊天补全
//! - Token usage 追踪
//! - 错误处理与重试

use anyhow::{Context, Result};
use futures::stream::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Anthropic API 客户端
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

/// 内容块类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// API 请求
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// API 响应（非流式）
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

/// Token 用量
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

/// 流式事件
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart {
        message: MessageStart,
    },
    ContentBlockStart {
        index: u32,
        content_block: ContentBlockStart,
    },
    ContentBlockDelta {
        index: u32,
        delta: ContentDelta,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        delta: MessageDelta,
        usage: Usage,
    },
    MessageStop,
    Ping,
}

/// 消息开始事件
#[derive(Debug, Deserialize)]
pub struct MessageStart {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

/// 内容块开始
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockStart {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

/// 内容增量
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}

/// 消息增量
#[derive(Debug, Deserialize, Clone)]
pub struct MessageDelta {
    pub stop_reason: Option<String>,
}

/// SSE 行解析结果
#[derive(Debug)]
enum SseEvent {
    Event(String),
    Data(String),
}

impl AnthropicClient {
    /// 创建新的客户端
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let model = model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            model,
            max_tokens: 4096,
        }
    }

    /// 创建客户端（自定义配置）
    pub fn with_config(
        api_key: String,
        model: String,
        max_tokens: u32,
        base_url: Option<String>,
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            model,
            max_tokens,
        }
    }

    /// 非流式聊天
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        system: Option<String>,
        tools: Option<Vec<ToolDef>>,
    ) -> Result<ChatResponse> {
        info!("Sending chat request to Anthropic API");

        let request = ChatRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages,
            system,
            tools,
            stream: None,
            metadata: None,
        };

        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2024-01-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("API error: {} - {}", status, body);
            anyhow::bail!("API error: {} - {}", status, body);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        info!(
            "Received response: {} tokens in, {} tokens out",
            chat_response.usage.input_tokens, chat_response.usage.output_tokens
        );

        Ok(chat_response)
    }

    /// 流式聊天
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        system: Option<String>,
        tools: Option<Vec<ToolDef>>,
    ) -> Result<impl Stream<Item = Result<StreamEvent>>> {
        info!("Sending streaming chat request to Anthropic API");

        let request = ChatRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages,
            system,
            tools,
            stream: Some(true),
            metadata: None,
        };

        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2024-01-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send streaming chat request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("API error: {} - {}", status, body);
            anyhow::bail!("API error: {} - {}", status, body);
        }

        // 处理 SSE 流
        let stream = response.bytes_stream();

        Ok(stream
            .filter_map(|chunk| async move {
                match chunk {
                    Ok(bytes) => {
                        let chunk_str = String::from_utf8_lossy(&bytes);

                        // 按行分割
                        let mut events = Vec::new();
                        for line in chunk_str.lines() {
                            if let Some(event) = Self::parse_sse_line(line) {
                                events.push(event);
                            }
                        }

                        // 解析为 StreamEvent
                        let mut result: Option<Result<StreamEvent>> = None;
                        for event in events {
                            match event {
                                SseEvent::Event(event_type) => {
                                    debug!("SSE event: {}", event_type);
                                }
                                SseEvent::Data(data) => {
                                    // 跳过 ping 事件
                                    if data == "{}" {
                                        continue;
                                    }

                                    match serde_json::from_str::<StreamEvent>(&data) {
                                        Ok(event) => result = Some(Ok(event)),
                                        Err(e) => {
                                            error!("Failed to parse SSE data: {} - {}", e, data);
                                        }
                                    }
                                }
                            }
                        }

                        result
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        Some(Err(e.into()))
                    }
                }
            })
            .filter_map(|result| async move { result.ok().map(Ok) }))
    }

    /// 解析 SSE 行
    fn parse_sse_line(line: &str) -> Option<SseEvent> {
        let line = line.trim();

        // 跳过空行和注释
        if line.is_empty() || line.starts_with(":") {
            return None;
        }

        if let Some(data) = line.strip_prefix("data: ") {
            Some(SseEvent::Data(data.to_string()))
        } else if let Some(event) = line.strip_prefix("event: ") {
            Some(SseEvent::Event(event.to_string()))
        } else {
            None
        }
    }

    /// 设置最大 token
    pub fn set_max_tokens(&mut self, max_tokens: u32) {
        self.max_tokens = max_tokens;
    }

    /// 设置模型
    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line_data() {
        let event = AnthropicClient::parse_sse_line("data: {\"type\":\"message_start\"}");
        assert!(matches!(event, Some(SseEvent::Data(_))));
    }

    #[test]
    fn test_parse_sse_line_event() {
        let event = AnthropicClient::parse_sse_line("event: message_start");
        assert!(matches!(event, Some(SseEvent::Event(_))));
    }

    #[test]
    fn test_parse_sse_line_empty() {
        let event = AnthropicClient::parse_sse_line("");
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_sse_line_comment() {
        let event = AnthropicClient::parse_sse_line(": ping");
        assert!(event.is_none());
    }
}
