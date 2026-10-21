use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

use crate::models::TokenUsage;
use crate::services::ai_provider::{AiModel, PromptSender};
use crate::services::http_client::{RetriableClient, RetryIntent};
use crate::services::provider_http::{
    ensure_success, exchange, log_status_if_error, ExchangeFailure, ExchangeSuccess,
    LOOKUP_BODY_TIMEOUT, LOOKUP_HEADER_TIMEOUT, PROBE_BODY_TIMEOUT, PROBE_HEADER_TIMEOUT,
};

/// Known Gemini models, ordered by priority (first = highest / default).
/// Fastest models come first so they are the default selection.
const KNOWN_GEMINI_MODELS: &[(&str, &str)] = &[
    ("gemini-2.5-flash-lite", "Gemini 2.5 Flash Lite"),
    ("gemini-2.5-flash", "Gemini 2.5 Flash"),
    ("gemini-3.1-flash-lite", "Gemini 3.1 Flash Lite"),
    ("gemini-3.1-flash-lite-preview", "Gemini 3.1 Flash Lite (Preview)"),
    ("gemini-3.5-flash", "Gemini 3.5 Flash"),
    ("gemini-3-flash-preview", "Gemini 3 Flash (Preview)"),
    ("gemini-2.5-pro", "Gemini 2.5 Pro"),
    ("gemini-3.1-pro-preview", "Gemini 3.1 Pro (Preview)"),
    ("gemini-2.0-flash", "Gemini 2.0 Flash (Deprecated)"),
    ("gemini-2.0-flash-lite", "Gemini 2.0 Flash Lite (Deprecated)"),
];

fn gemini_priority(id: &str) -> u32 {
    KNOWN_GEMINI_MODELS
        .iter()
        .position(|(known_id, _)| *known_id == id)
        .map(|pos| pos as u32)
        .unwrap_or(999)
}

pub struct GeminiService {
    api_key: String,
    model: String,
    client: RetriableClient,
}

impl GeminiService {
    fn supports_generate_content(model: &GeminiModelInfo) -> bool {
        model
            .supported_generation_methods
            .as_ref()
            .map(|methods| methods.iter().any(|method| method == "generateContent"))
            .unwrap_or(false)
    }

    fn ai_model_from_api(id: String, info: GeminiModelInfo) -> AiModel {
        AiModel {
            name: info.display_name.unwrap_or(id.clone()),
            id,
        }
    }

    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: RetriableClient::default(),
        }
    }

    fn get_api_url(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        )
    }

    fn list_models_url(api_key: &str) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key
        )
    }

    pub(crate) async fn fetch_models_internal(api_key: &str) -> Result<Vec<AiModel>> {
        debug!("Fetching Gemini models");

        let client = RetriableClient::default();
        let url = Self::list_models_url(api_key);
        let response = client
            .send_with_retry("gemini.models", RetryIntent::Idempotent, move |http| {
                let url = url.clone();
                let client = http.clone();
                async move { client.get(&url).send().await }
            })
            .await?;

        let response = ensure_success("Gemini models", response).await?;
        let models_response: GeminiModelsResponse = response.json().await?;

        let mut api_models: HashMap<String, GeminiModelInfo> = HashMap::new();
        for model in models_response.models {
            if !Self::supports_generate_content(&model) {
                continue;
            }

            let model_id = model
                .name
                .strip_prefix("models/")
                .unwrap_or(&model.name)
                .to_string();
            api_models.insert(model_id, model);
        }

        let mut models: Vec<AiModel> = Vec::new();
        for &(id, name) in KNOWN_GEMINI_MODELS {
            if api_models.remove(id).is_some() {
                models.push(AiModel { id: id.to_string(), name: name.to_string() });
            }
        }

        for (id, info) in api_models {
            models.push(Self::ai_model_from_api(id, info));
        }

        models.sort_by_key(|m| gemini_priority(&m.id));

        info!("Successfully fetched {} Gemini models", models.len());
        Ok(models)
    }

    pub(crate) async fn test_api_key_internal(api_key: &str) -> Result<bool> {
        debug!("Testing Gemini API key");

        let client = RetriableClient::default();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite:generateContent?key={}",
            api_key
        );

        let test_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": "test"
                }]
            }],
            "generationConfig": {
                "maxOutputTokens": 10
            }
        });

        let response = client
            .send_with_retry(
                "gemini.validate_key",
                RetryIntent::Idempotent,
                move |http| {
                    let url = url.clone();
                    let test_body = test_body.clone();
                    let client = http.clone();
                    async move { client.post(&url).json(&test_body).send().await }
                },
            )
            .await?;

        let status = response.status();
        log_status_if_error("Gemini API key", status);
        let is_valid = status.is_success();
        debug!("Gemini API key test result: {}", is_valid);
        Ok(is_valid)
    }

    pub async fn probe_completion(
        api_key: &str,
        model: &str,
    ) -> Result<ExchangeSuccess, ExchangeFailure> {
        let service = Self::new(api_key.to_string(), model.to_string());
        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: "Reply with the single word pong.".to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                max_output_tokens: 32,
                response_mime_type: None,
                thinking_config: Some(ThinkingConfig { thinking_budget: 0 }),
            },
        };
        let url = service.get_api_url();
        exchange(
            "Gemini probe",
            PROBE_HEADER_TIMEOUT,
            PROBE_BODY_TIMEOUT,
            service
                .client
                .send_with_retry("gemini.probe", RetryIntent::NonIdempotent, |http| {
                    let request_body = request_body.clone();
                    let url = url.clone();
                    let client = http.clone();
                    async move { client.post(&url).json(&request_body).send().await }
                }),
        )
        .await
    }
}

#[async_trait]
impl PromptSender for GeminiService {
    async fn send_prompt(&self, prompt: &str) -> Result<(String, Option<TokenUsage>)> {
        info!("Sending prompt to Gemini (model: {})", self.model);
        debug!("Prompt length: {} chars", prompt.len());

        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                max_output_tokens: 8000,
                response_mime_type: Some("application/json".to_string()),
                thinking_config: Some(ThinkingConfig { thinking_budget: 0 }),
            },
        };

        let url = self.get_api_url();
        let posted = exchange(
            "Gemini generate",
            LOOKUP_HEADER_TIMEOUT,
            LOOKUP_BODY_TIMEOUT,
            self.client
                .send_with_retry("gemini.generate", RetryIntent::NonIdempotent, |http| {
                    let request_body = request_body.clone();
                    let url = url.clone();
                    let client = http.clone();
                    async move { client.post(&url).json(&request_body).send().await }
                }),
        )
        .await
        .map_err(ExchangeFailure::into_anyhow)?;
        let text = posted.body;
        debug!(
            "Gemini response status: {} length: {} bytes",
            posted.status,
            text.len()
        );

        if let Ok(gemini_resp) = serde_json::from_str::<GeminiResponse>(&text) {
            if let Some(candidate) = gemini_resp.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    info!("Successfully parsed Gemini response");

                    let token_usage = gemini_resp.usage_metadata.map(|usage| TokenUsage {
                        prompt_tokens: usage.prompt_token_count,
                        completion_tokens: usage.candidates_token_count,
                        total_tokens: usage.total_token_count,
                    });

                    if let Some(ref tokens) = token_usage {
                        debug!(
                            "Token usage - input: {}, output: {}, total: {}",
                            tokens.prompt_tokens, tokens.completion_tokens, tokens.total_tokens
                        );
                    }

                    return Ok((part.text.clone(), token_usage));
                }
            }
        }

        error!("Failed to parse Gemini response structure");
        Err(anyhow!("Failed to parse Gemini response: {}", text))
    }
}

// Gemini API request/response structures

#[derive(Debug, Serialize, Clone)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize, Clone)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Clone)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize, Clone)]
struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: u32,
}

#[derive(Debug, Serialize, Clone)]
struct GenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    #[serde(rename = "responseMimeType", skip_serializing_if = "Option::is_none")]
    response_mime_type: Option<String>,
    #[serde(rename = "thinkingConfig", skip_serializing_if = "Option::is_none")]
    thinking_config: Option<ThinkingConfig>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}

#[derive(Debug, Deserialize)]
struct GeminiModelsResponse {
    models: Vec<GeminiModelInfo>,
}

#[derive(Debug, Deserialize)]
struct GeminiModelInfo {
    name: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "supportedGenerationMethods")]
    supported_generation_methods: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gemini_api_url_format() {
        let service = GeminiService::new("test-key".to_string(), "gemini-2.5-flash-lite".to_string());
        let url = service.get_api_url();
        assert!(url.contains("generativelanguage.googleapis.com"));
        assert!(url.contains("gemini-2.5-flash-lite"));
        assert!(url.contains("generateContent"));
    }

    #[tokio::test]
    async fn parse_gemini_response_format() {
        let gemini_response = r#"{
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": "{\"word\":\"test\",\"phonetic\":\"/tɛst/\",\"meanings\":[]}"
                    }],
                    "role": "model"
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 100,
                "candidatesTokenCount": 50,
                "totalTokenCount": 150
            }
        }"#;

        let response: GeminiResponse = serde_json::from_str(gemini_response).unwrap();
        assert_eq!(response.candidates.len(), 1);
        assert!(response.candidates[0].content.parts[0]
            .text
            .contains("test"));
    }

    #[test]
    fn gemini_known_models_have_friendly_names() {
        // Look up by id rather than position so the test survives list reordering.
        let entry = KNOWN_GEMINI_MODELS
            .iter()
            .find(|(id, _)| *id == "gemini-2.5-pro");
        assert_eq!(entry, Some(&("gemini-2.5-pro", "Gemini 2.5 Pro")));
    }
}
