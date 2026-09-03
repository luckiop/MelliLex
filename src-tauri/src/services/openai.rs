use crate::constants::api::{deepseek as deepseek_api, openai as openai_api};
use crate::models::TokenUsage;
use crate::services::ai_provider::{AiModel, PromptSender};
use crate::services::http_client::{RetriableClient, RetryIntent};
use crate::services::model_info::ModelInfoService;
use crate::services::provider_http::{ensure_success, log_status_if_error};
use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
enum CompatKind {
    OpenAi,
    DeepSeek,
}

impl CompatKind {
    fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::DeepSeek => "DeepSeek",
        }
    }

    fn chat_url(self) -> &'static str {
        match self {
            Self::OpenAi => openai_api::CHAT_COMPLETIONS,
            Self::DeepSeek => deepseek_api::CHAT_COMPLETIONS,
        }
    }

    fn models_url(self) -> &'static str {
        match self {
            Self::OpenAi => openai_api::MODELS,
            Self::DeepSeek => deepseek_api::MODELS,
        }
    }

    fn retry_prefix(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::DeepSeek => "deepseek",
        }
    }
}

pub struct OpenAIService {
    client: RetriableClient,
    api_key: String,
    model: String,
    kind: CompatKind,
}

impl OpenAIService {
    pub fn new(api_key: String, model: String) -> Self {
        Self::with_kind(api_key, model, CompatKind::OpenAi)
    }

    pub fn deepseek(api_key: String, model: String) -> Self {
        Self::with_kind(api_key, model, CompatKind::DeepSeek)
    }

    fn with_kind(api_key: String, model: String, kind: CompatKind) -> Self {
        OpenAIService {
            client: RetriableClient::default(),
            api_key,
            model,
            kind,
        }
    }

    pub(crate) async fn fetch_models_internal(api_key: &str) -> Result<Vec<AiModel>> {
        Self::fetch_models_for(CompatKind::OpenAi, api_key).await
    }

    pub(crate) async fn fetch_deepseek_models(api_key: &str) -> Result<Vec<AiModel>> {
        Self::fetch_models_for(CompatKind::DeepSeek, api_key).await
    }

    async fn fetch_models_for(kind: CompatKind, api_key: &str) -> Result<Vec<AiModel>> {
        debug!("Fetching {} models", kind.label());

        let models_url = kind.models_url();
        let retry_key = format!("{}.models", kind.retry_prefix());
        let client = RetriableClient::default();
        let response = client
            .send_with_retry(&retry_key, RetryIntent::Idempotent, |http| {
                let api_key = api_key.to_string();
                let client = http.clone();
                async move {
                    client
                        .get(models_url)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await
                }
            })
            .await?;

        let response = ensure_success(&format!("{} models", kind.label()), response).await?;
        let models_response: OpenAiModelsResponse = response.json().await?;

        let total_count = models_response.data.len();
        debug!("Total models from API: {}", total_count);

        let mut models: Vec<AiModel> = models_response
            .data
            .into_iter()
            .filter(|m| is_useful_model(kind, &m.id))
            .map(|m| match kind {
                CompatKind::OpenAi => ModelInfoService::create_ai_model(m.id),
                CompatKind::DeepSeek => AiModel {
                    name: deepseek_display_name(&m.id),
                    id: m.id,
                },
            })
            .collect();

        models.sort_by_key(|m| (model_sort_key(kind, &m.id), m.id.len()));

        let mut seen_names = std::collections::HashSet::new();
        models.retain(|m| seen_names.insert(m.name.clone()));

        info!(
            "Successfully fetched {} {} chat models (filtered from {} total)",
            models.len(),
            kind.label(),
            total_count
        );
        Ok(models)
    }

    pub(crate) async fn test_api_key_internal(api_key: &str) -> Result<bool> {
        Self::test_api_key_for(CompatKind::OpenAi, api_key).await
    }

    pub(crate) async fn test_deepseek_api_key(api_key: &str) -> Result<bool> {
        Self::test_api_key_for(CompatKind::DeepSeek, api_key).await
    }

    async fn test_api_key_for(kind: CompatKind, api_key: &str) -> Result<bool> {
        debug!("Testing {} API key", kind.label());

        let models_url = kind.models_url();
        let retry_key = format!("{}.validate_key", kind.retry_prefix());
        let client = RetriableClient::default();
        let response = client
            .send_with_retry(&retry_key, RetryIntent::Idempotent, |http| {
                let api_key = api_key.to_string();
                let client = http.clone();
                async move {
                    client
                        .get(models_url)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await
                }
            })
            .await?;

        let status = response.status();
        log_status_if_error(&format!("{} API key", kind.label()), status);
        let is_valid = status.is_success();
        debug!("{} API key test result: {}", kind.label(), is_valid);
        Ok(is_valid)
    }

    fn request_body(&self, prompt: &str) -> serde_json::Value {
        match self.kind {
            CompatKind::OpenAi => serde_json::json!({
                "model": self.model,
                "messages": [
                    { "role": "user", "content": prompt }
                ],
                "max_completion_tokens": 16000,
                "response_format": { "type": "json_object" }
            }),
            CompatKind::DeepSeek => serde_json::json!({
                "model": self.model,
                "messages": [
                    { "role": "user", "content": prompt }
                ],
                "max_tokens": 1200,
                "thinking": { "type": "disabled" },
                "response_format": { "type": "json_object" }
            }),
        }
    }
}

fn is_useful_model(kind: CompatKind, id: &str) -> bool {
    match kind {
        CompatKind::OpenAi => ModelInfoService::is_useful_chat_model(id),
        CompatKind::DeepSeek => {
            let normalized = id.trim().to_ascii_lowercase();
            normalized.contains("deepseek")
        }
    }
}

fn model_sort_key(kind: CompatKind, id: &str) -> u32 {
    match kind {
        CompatKind::OpenAi => ModelInfoService::get_model_priority(id),
        CompatKind::DeepSeek => {
            let normalized = id.to_ascii_lowercase();
            if normalized.contains("flash") {
                0
            } else if normalized.contains("pro") {
                1
            } else {
                10
            }
        }
    }
}

fn deepseek_display_name(id: &str) -> String {
    match id {
        "deepseek-v4-flash" => "DeepSeek V4 Flash".to_string(),
        "deepseek-v4-pro" => "DeepSeek V4 Pro".to_string(),
        other => other.to_string(),
    }
}

#[async_trait]
impl PromptSender for OpenAIService {
    async fn send_prompt(&self, prompt: &str) -> Result<(String, Option<TokenUsage>)> {
        let label = self.kind.label();
        log::info!("Sending prompt to {} (model: {})", label, self.model);
        log::debug!("Prompt length: {} characters", prompt.len());

        let body = self.request_body(prompt);
        let api_key = self.api_key.clone();
        let chat_url = self.kind.chat_url();
        let retry_key = format!("{}.chat_completions", self.kind.retry_prefix());
        let resp = self
            .client
            .send_with_retry(&retry_key, RetryIntent::NonIdempotent, |http| {
                let body = body.clone();
                let api_key = api_key.clone();
                let client = http.clone();
                async move {
                    client
                        .post(chat_url)
                        .bearer_auth(api_key)
                        .json(&body)
                        .send()
                        .await
                }
            })
            .await
            .map_err(|e| {
                log::error!("{} HTTP request failed: {}", label, e);
                e
            })?;

        let status = resp.status();
        log::debug!("{} response status: {}", label, status);

        let resp = ensure_success(&format!("{} chat completions", label), resp).await?;
        let text = resp.text().await?;
        log::debug!("{} response length: {} bytes", label, text.len());

        if let Ok(v) = serde_json::from_str::<OpenAIResponse>(&text) {
            if let Some(choice) = v.choices.first() {
                let content = choice.message.content.clone().unwrap_or_default();
                log::info!("Successfully parsed {} response", label);
                log::debug!("{} content length: {} chars", label, content.len());

                if content.is_empty() {
                    let reasoning = choice
                        .message
                        .reasoning_content
                        .as_deref()
                        .unwrap_or_default();
                    log::error!(
                        "{} returned empty content (reasoning_content {} chars). Raw response: {}",
                        label,
                        reasoning.len(),
                        &text.chars().take(1000).collect::<String>()
                    );
                    return Err(anyhow::anyhow!(
                        "{} returned empty content for model {} (thinking produced no JSON)",
                        label,
                        self.model
                    ));
                }

                let token_usage = v.usage.map(|usage| TokenUsage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                });

                if let Some(ref tokens) = token_usage {
                    log::debug!(
                        "Token usage - prompt: {}, completion: {}, total: {}",
                        tokens.prompt_tokens,
                        tokens.completion_tokens,
                        tokens.total_tokens
                    );
                }

                return Ok((content, token_usage));
            }
        }

        log::error!("Failed to parse {} response structure", label);
        Err(anyhow::anyhow!("Failed to parse {} response: {}", label, text))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelInfo {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_openai_response_format() {
        let openai_response = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "{\"word\":\"test\",\"phonetic\":\"/tɛst/\",\"meanings\":[{\"partOfSpeech\":\"noun\",\"definitions\":[\"A procedure\"]}],\"synonyms\":[],\"antonyms\":[],\"examples\":[]}"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            }
        }"#;

        let response: OpenAIResponse = serde_json::from_str(openai_response).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert!(response.choices[0]
            .message
            .content
            .as_deref()
            .unwrap_or_default()
            .contains("test"));
    }

    #[test]
    fn deepseek_keeps_flash_models() {
        assert!(is_useful_model(CompatKind::DeepSeek, "deepseek-v4-flash"));
        assert!(!is_useful_model(CompatKind::DeepSeek, "gpt-5-mini"));
        assert_eq!(model_sort_key(CompatKind::DeepSeek, "deepseek-v4-flash"), 0);
    }
}
