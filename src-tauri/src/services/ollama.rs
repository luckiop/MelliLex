use crate::errors::AppError;
use crate::models::TokenUsage;
use crate::services::ai_provider::{AiModel, PromptSender};
use crate::services::provider_http::{
    exchange, ExchangeFailure, ExchangeSuccess, LOOKUP_BODY_TIMEOUT, LOOKUP_HEADER_TIMEOUT,
    PROBE_BODY_TIMEOUT, PROBE_HEADER_TIMEOUT,
};
use anyhow::Result;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::Deserialize;

pub struct OllamaService {
    client: Client,
    endpoint: String,
    model: String,
}

impl OllamaService {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            client: Client::new(),
            endpoint: normalize_endpoint(Some(&endpoint)),
            model,
        }
    }

    pub fn for_detection() -> Self {
        Self {
            client: Client::new(),
            endpoint: normalize_endpoint(None),
            model: String::new(),
        }
    }

    fn generate_url(&self) -> String {
        format!("{}/api/generate", self.endpoint)
    }

    pub(crate) async fn fetch_models_internal(endpoint: &str) -> Result<Vec<AiModel>> {
        debug!("Fetching Ollama models from {}", endpoint);

        let client = Client::new();
        let response = client.get(tags_url(endpoint)).send().await?;

        if !response.status().is_success() {
            error!("Ollama API error: status {}", response.status());
            return Err(anyhow::anyhow!("Failed to fetch models from Ollama"));
        }

        let payload: OllamaTagsResponse = response.json().await?;
        let names = payload.model_names();

        let models: Vec<AiModel> = names
            .into_iter()
            .map(|name| AiModel {
                id: name.clone(),
                name,
            })
            .collect();

        info!("Successfully fetched {} Ollama models", models.len());
        Ok(models)
    }

    pub(crate) async fn test_endpoint_internal(endpoint: &str) -> Result<bool> {
        debug!("Testing Ollama endpoint: {}", endpoint);

        let client = Client::new();
        match client.get(tags_url(endpoint)).send().await {
            Ok(resp) => {
                let is_valid = resp.status().is_success();
                debug!("Ollama endpoint test result: {}", is_valid);
                Ok(is_valid)
            }
            Err(e) => {
                debug!("Failed to reach Ollama at {}: {}", endpoint, e);
                Ok(false)
            }
        }
    }

    pub async fn probe_completion(
        endpoint: &str,
        model: &str,
    ) -> Result<ExchangeSuccess, ExchangeFailure> {
        let service = Self::new(endpoint.to_string(), model.to_string());
        let request_payload = serde_json::json!({
            "model": service.model,
            "prompt": "Reply with the single word pong.",
            "stream": false,
        });
        let url = service.generate_url();
        exchange(
            "Ollama probe",
            PROBE_HEADER_TIMEOUT,
            PROBE_BODY_TIMEOUT,
            service.client.post(&url).json(&request_payload).send(),
        )
        .await
    }

    pub async fn detect(&self, endpoint: &str) -> bool {
        match self.client.get(tags_url(endpoint)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(err) => {
                debug!("Failed to reach Ollama at {}: {}", endpoint, err);
                false
            }
        }
    }

    pub async fn list_models(&self, endpoint: &str) -> Result<Vec<String>, AppError> {
        let response = self
            .client
            .get(tags_url(endpoint))
            .send()
            .await
            .map_err(AppError::from)?;

        if !response.status().is_success() {
            warn!(
                "Ollama responded with status {} when listing models",
                response.status()
            );
        }

        let payload: OllamaTagsResponse = response.json().await.map_err(AppError::from)?;
        Ok(payload.model_names())
    }

    pub async fn fetch_ai_models(&self, endpoint: &str) -> Result<Vec<AiModel>, AppError> {
        let names = self.list_models(endpoint).await?;
        let models = names
            .into_iter()
            .map(|name| AiModel {
                id: name.clone(),
                name,
            })
            .collect();
        Ok(models)
    }
}

fn normalize_endpoint(raw: Option<&str>) -> String {
    let default = "http://localhost:11434";
    let trimmed = raw.unwrap_or(default).trim();
    let without_trailing = trimmed.trim_end_matches('/');
    if without_trailing.is_empty() {
        default.to_string()
    } else {
        without_trailing.to_string()
    }
}

pub fn normalize_ollama_endpoint(raw: Option<&str>) -> String {
    normalize_endpoint(raw)
}

fn tags_url(endpoint: &str) -> String {
    format!("{}/api/tags", endpoint)
}

#[async_trait]
impl PromptSender for OllamaService {
    async fn send_prompt(&self, prompt: &str) -> Result<(String, Option<TokenUsage>)> {
        debug!(
            "Sending prompt to Ollama: model={}, endpoint={}",
            self.model, self.endpoint
        );

        let request_payload = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
        });

        let url = self.generate_url();
        let posted = exchange(
            "Ollama generate",
            LOOKUP_HEADER_TIMEOUT,
            LOOKUP_BODY_TIMEOUT,
            self.client.post(&url).json(&request_payload).send(),
        )
        .await
        .map_err(ExchangeFailure::into_anyhow)?;

        let payload: serde_json::Value = serde_json::from_str(&posted.body)?;

        let content = payload["response"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'response' field in Ollama response"))?
            .to_string();

        let token_usage = if let (Some(prompt_tokens), Some(completion_tokens)) = (
            payload["prompt_eval_count"].as_u64(),
            payload["eval_count"].as_u64(),
        ) {
            Some(TokenUsage {
                prompt_tokens: prompt_tokens as u32,
                completion_tokens: completion_tokens as u32,
                total_tokens: (prompt_tokens + completion_tokens) as u32,
            })
        } else {
            None
        };

        debug!("Received Ollama response: {} chars", content.len());
        Ok((content, token_usage))
    }
}

impl Default for OllamaService {
    fn default() -> Self {
        Self::for_detection()
    }
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModelInfo>>,
}

impl OllamaTagsResponse {
    fn model_names(self) -> Vec<String> {
        self.models
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.name)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_empty_endpoint_to_default() {
        assert_eq!(normalize_endpoint(Some("   ")), "http://localhost:11434");
    }

    #[test]
    fn strips_trailing_slash() {
        assert_eq!(
            normalize_endpoint(Some("http://host:1234/")),
            "http://host:1234"
        );
    }

    #[test]
    fn keeps_valid_endpoint() {
        assert_eq!(
            normalize_endpoint(Some("http://custom:9000")),
            "http://custom:9000"
        );
    }

    #[test]
    fn extracts_model_names_from_response() {
        let payload = OllamaTagsResponse {
            models: Some(vec![
                OllamaModelInfo {
                    name: Some("llama3".into()),
                },
                OllamaModelInfo { name: None },
                OllamaModelInfo {
                    name: Some("phi3".into()),
                },
            ]),
        };

        let names = payload.model_names();
        assert_eq!(names, vec!["llama3", "phi3"]);
    }
}
