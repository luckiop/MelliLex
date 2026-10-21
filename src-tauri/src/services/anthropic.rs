use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::constants::api::anthropic as anthropic_api;
use crate::constants::headers;
use crate::models::TokenUsage;
use crate::services::ai_provider::{AiModel, PromptSender};
use crate::services::http_client::{RetriableClient, RetryIntent};
use crate::services::provider_http::{
    exchange, ExchangeFailure, ExchangeSuccess, LOOKUP_BODY_TIMEOUT, LOOKUP_HEADER_TIMEOUT,
    PROBE_BODY_TIMEOUT, PROBE_HEADER_TIMEOUT,
};

pub struct AnthropicService {
    api_key: String,
    model: String,
    client: RetriableClient,
}

impl AnthropicService {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: RetriableClient::default(),
        }
    }

    pub(crate) async fn fetch_models_internal(api_key: &str) -> Result<Vec<AiModel>> {
        debug!("Fetching Anthropic models (curated list)");

        if !Self::test_api_key_internal(api_key).await? {
            return Err(anyhow::anyhow!("Invalid API key"));
        }

        let models = KNOWN_ANTHROPIC_MODELS
            .iter()
            .map(|&(id, name)| AiModel {
                id: id.to_string(),
                name: name.to_string(),
            })
            .collect::<Vec<_>>();

        info!("Successfully fetched {} Anthropic models", models.len());
        Ok(models)
    }

    pub(crate) async fn test_api_key_internal(api_key: &str) -> Result<bool> {
        debug!("Testing Anthropic API key");

        let client = RetriableClient::default();

        let response = client
            .send_with_retry("anthropic.validate_key", RetryIntent::Idempotent, |http| {
                let api_key = api_key.to_string();
                let client = http.clone();
                async move {
                    client
                        .get(anthropic_api::MODELS)
                        .header(headers::X_API_KEY, api_key)
                        .header(headers::ANTHROPIC_VERSION, anthropic_api::VERSION)
                        .send()
                        .await
                }
            })
            .await?;

        let status = response.status();
        let is_valid = status.is_success();
        if !is_valid {
            let body = response.text().await.unwrap_or_default();
            log::error!("Anthropic API key error: status {}, body: {}", status, body);
        }
        debug!("Anthropic API key test result: {}", is_valid);
        Ok(is_valid)
    }

    pub async fn probe_completion(
        api_key: &str,
        model: &str,
    ) -> Result<ExchangeSuccess, ExchangeFailure> {
        let service = Self::new(api_key.to_string(), model.to_string());
        let request_body = AnthropicRequest {
            model: service.model.clone(),
            max_tokens: 32,
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![MessageContent {
                    content_type: "text".to_string(),
                    text: "Reply with the single word pong.".to_string(),
                }],
            }],
        };
        let api_key = service.api_key.clone();
        exchange(
            "Anthropic probe",
            PROBE_HEADER_TIMEOUT,
            PROBE_BODY_TIMEOUT,
            service
                .client
                .send_with_retry("anthropic.probe", RetryIntent::NonIdempotent, |http| {
                    let request_body = request_body.clone();
                    let api_key = api_key.clone();
                    let client = http.clone();
                    async move {
                        client
                            .post(anthropic_api::MESSAGES)
                            .header(headers::X_API_KEY, api_key)
                            .header(headers::ANTHROPIC_VERSION, anthropic_api::VERSION)
                            .header(headers::CONTENT_TYPE, "application/json")
                            .json(&request_body)
                            .send()
                            .await
                    }
                }),
        )
        .await
    }
}

#[async_trait]
impl PromptSender for AnthropicService {
    async fn send_prompt(&self, prompt: &str) -> Result<(String, Option<TokenUsage>)> {
        self.generate_completion(prompt).await
    }
}

impl AnthropicService {
    async fn generate_completion(&self, prompt: &str) -> Result<(String, Option<TokenUsage>)> {
        info!("Sending prompt to Anthropic (model: {})", self.model);
        debug!("Prompt length: {} chars", prompt.len());

        let request_body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 8000,
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![MessageContent {
                    content_type: "text".to_string(),
                    text: prompt.to_string(),
                }],
            }],
        };

        let api_key = self.api_key.clone();
        let posted = exchange(
            "Anthropic messages",
            LOOKUP_HEADER_TIMEOUT,
            LOOKUP_BODY_TIMEOUT,
            self.client
                .send_with_retry("anthropic.messages", RetryIntent::NonIdempotent, |http| {
                    let request_body = request_body.clone();
                    let api_key = api_key.clone();
                    let client = http.clone();
                    async move {
                        client
                            .post(anthropic_api::MESSAGES)
                            .header(headers::X_API_KEY, api_key)
                            .header(headers::ANTHROPIC_VERSION, anthropic_api::VERSION)
                            .header(headers::CONTENT_TYPE, "application/json")
                            .json(&request_body)
                            .send()
                            .await
                    }
                }),
        )
        .await
        .map_err(ExchangeFailure::into_anyhow)?;
        let api_response: AnthropicResponse = serde_json::from_str(&posted.body)?;

        let token_usage = api_response.usage.map(|usage| TokenUsage {
            prompt_tokens: usage.input_tokens,
            completion_tokens: usage.output_tokens,
            total_tokens: usage.input_tokens + usage.output_tokens,
        });

        if let Some(ref tokens) = token_usage {
            debug!(
                "Token usage - input: {}, output: {}, total: {}",
                tokens.prompt_tokens, tokens.completion_tokens, tokens.total_tokens
            );
        }

        if let Some(content_block) = api_response
            .content
            .iter()
            .find(|block| block.content_type == "text")
        {
            info!("Successfully received response from Anthropic");
            debug!("Response length: {} chars", content_block.text.len());
            return Ok((content_block.text.clone(), token_usage));
        }

        Err(anyhow!("No text content in Anthropic response"))
    }
}

#[derive(Serialize, Clone)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize, Clone)]
struct Message {
    role: String,
    content: Vec<MessageContent>,
}

#[derive(Serialize, Clone)]
struct MessageContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Known Anthropic models, ordered by priority (first = highest / default).
/// Fastest models come first so they are the default selection.
const KNOWN_ANTHROPIC_MODELS: &[(&str, &str)] = &[
    ("claude-haiku-4-5-20251001", "Claude Haiku 4.5"),
    ("claude-sonnet-4-6", "Claude Sonnet 4.6"),
    ("claude-opus-4-7", "Claude Opus 4.7"),
    ("claude-sonnet-4-5-20250929", "Claude Sonnet 4.5"),
    ("claude-opus-4-6", "Claude Opus 4.6"),
    ("claude-opus-4-5-20251101", "Claude Opus 4.5"),
    ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku"),
    ("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet (October 2024)"),
    ("claude-3-5-sonnet-20240620", "Claude 3.5 Sonnet (June 2024)"),
    ("claude-3-haiku-20240307", "Claude 3 Haiku"),
    ("claude-3-sonnet-20240229", "Claude 3 Sonnet"),
    ("claude-3-opus-20240229", "Claude 3 Opus"),
];
