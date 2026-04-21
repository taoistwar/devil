//! OpenAI API 客户端
//!
//! 实现：
//! - 流式聊天补全
//! - Token usage 追踪
//! - 错误处理与重试

use anyhow::Result;
use futures::stream::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// OpenAI API 客户端
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
}

/// OpenAI 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI 内容块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContentBlock {
    Text {
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        #[serde(rename = "function")]
        function: OpenAIFunction,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        #[serde(rename = "tool_use_id")]
        tool_use_id: String,
        content: String,
    },
}

/// OpenAI 函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub arguments: String,
}

/// OpenAI 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolDef {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunctionDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// OpenAI API 请求
#[derive(Debug, Serialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<OpenAIChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAIToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// OpenAI API 响应（非流式）
#[derive(Debug, Deserialize)]
pub struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

/// OpenAI Choice
#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIMessage,
    pub finish_reason: String,
}

/// OpenAI Message
#[derive(Debug, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI Usage
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 流式事件
#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub enum OpenAIStreamEvent {
    #[serde(rename = "thread.created")]
    ThreadCreated,
    #[serde(rename = "thread.run.created")]
    ThreadRunCreated,
    #[serde(rename = "thread.run.completed")]
    ThreadRunCompleted,
    #[serde(rename = "message.created")]
    MessageCreated,
    #[serde(rename = "message.delta")]
    MessageDelta,
}

impl OpenAIClient {
    /// 创建新的 OpenAI 客户端
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com".to_string(),
            model,
            max_tokens: 4096,
        }
    }

    /// 创建带有自定义配置的客户端
    pub fn with_config(api_key: String, base_url: String, model: String, max_tokens: u32) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
            model,
            max_tokens,
        }
    }

    /// 设置基础 URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// 设置最大输出 tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// 发送聊天请求（非流式）
    pub async fn chat(&self, messages: Vec<OpenAIChatMessage>) -> Result<OpenAIChatResponse> {
        let request = OpenAIChatRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages,
            system: None,
            tools: None,
            stream: Some(false),
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {} - {}", status, text);
            anyhow::bail!("OpenAI API error: {} - {}", status, text);
        }

        let chat_response: OpenAIChatResponse = response.json().await?;
        debug!(
            "OpenAI response: {} tokens (prompt: {}, completion: {})",
            chat_response.usage.total_tokens,
            chat_response.usage.prompt_tokens,
            chat_response.usage.completion_tokens
        );

        Ok(chat_response)
    }

    /// 发送聊天请求（流式）
    #[allow(dead_code)]
    pub async fn chat_streaming(
        &self,
        messages: Vec<OpenAIChatMessage>,
    ) -> Result<impl Stream<Item = Result<String>>> {
        let request = OpenAIChatRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages,
            system: None,
            tools: None,
            stream: Some(true),
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {} - {}", status, text);
            anyhow::bail!("OpenAI API error: {} - {}", status, text);
        }

        let stream = response.bytes_stream().map(|chunk| {
            match chunk {
                Ok(bytes) => {
                    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                        Ok(s)
                    } else {
                        Ok(String::new())
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Stream error: {}", e)),
            }
        });

        Ok(stream)
    }

    /// 列出可用模型
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/v1/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {} - {}", status, text);
            anyhow::bail!("OpenAI API error: {} - {}", status, text);
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<Model>,
        }

        #[derive(Deserialize)]
        struct Model {
            id: String,
        }

        let models: ModelsResponse = response.json().await?;
        Ok(models.data.into_iter().map(|m| m.id).collect())
    }
}

impl OpenAIClient {
    /// 获取客户端版本信息
    pub fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new(
            "test-api-key".to_string(),
            "gpt-4".to_string(),
        );
        assert_eq!(client.model, "gpt-4");
    }

    #[test]
    fn test_openai_client_with_config() {
        let client = OpenAIClient::with_config(
            "test-api-key".to_string(),
            "https://api.openai.com".to_string(),
            "gpt-4-turbo".to_string(),
            8192,
        );
        assert_eq!(client.model, "gpt-4-turbo");
        assert_eq!(client.max_tokens, 8192);
    }
}
