use serde::Serialize;

use crate::constants::providers;
use crate::services::anthropic::AnthropicService;
use crate::services::gemini::GeminiService;
use crate::services::ollama::{normalize_ollama_endpoint, OllamaService};
use crate::services::openai::OpenAIService;
use crate::services::provider_http::{
    redact_secrets, truncate_chars, ExchangeFailure, ExchangeSuccess,
};

pub const REASON_OK: &str = "ok";
pub const REASON_NOT_CONFIGURED: &str = "not_configured";
pub const REASON_TIMEOUT_AFTER_HEADERS: &str = "timeout_after_headers";
pub const REASON_TIMEOUT: &str = "timeout";
pub const REASON_NETWORK: &str = "network";
pub const REASON_HTTP_ERROR: &str = "http_error";
pub const REASON_EMPTY_REPLY: &str = "empty_reply";
pub const REASON_UNSUPPORTED: &str = "unsupported";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectivityReport {
    pub ok: bool,
    pub reason: String,
    pub elapsed_ms: u64,
    pub http_status: Option<u16>,
    pub provider: String,
    pub model: String,
    pub reply_preview: Option<String>,
    pub raw_error: String,
}

pub async fn probe_provider(provider: &str, credential: &str, model: &str) -> ConnectivityReport {
    let provider = provider.trim();
    let model = model.trim();
    let credential = credential.trim();

    if model.is_empty() || (provider != providers::OLLAMA && credential.is_empty()) {
        return blank_report(
            provider,
            model,
            REASON_NOT_CONFIGURED,
            "API key or model is empty",
        );
    }

    let outcome = match provider {
        providers::OPENAI => OpenAIService::probe_openai(credential, model).await,
        providers::DEEPSEEK => OpenAIService::probe_deepseek(credential, model).await,
        providers::ANTHROPIC => AnthropicService::probe_completion(credential, model).await,
        providers::GEMINI => GeminiService::probe_completion(credential, model).await,
        providers::OLLAMA => {
            let endpoint = if credential.is_empty() {
                normalize_ollama_endpoint(None)
            } else {
                normalize_ollama_endpoint(Some(credential))
            };
            OllamaService::probe_completion(&endpoint, model).await
        }
        _ => {
            return blank_report(
                provider,
                model,
                REASON_UNSUPPORTED,
                &format!("Provider '{provider}' is not supported"),
            );
        }
    };

    let extract: fn(&str) -> Option<String> = match provider {
        providers::ANTHROPIC => extract_anthropic_reply,
        providers::GEMINI => extract_gemini_reply,
        providers::OLLAMA => extract_ollama_reply,
        _ => extract_openai_reply,
    };
    into_report(provider, model, outcome, extract)
}

pub fn into_report(
    provider: &str,
    model: &str,
    outcome: Result<ExchangeSuccess, ExchangeFailure>,
    extract_reply: impl Fn(&str) -> Option<String>,
) -> ConnectivityReport {
    match outcome {
        Ok(success) => match extract_reply(&success.body) {
            Some(reply) => ConnectivityReport {
                ok: true,
                reason: REASON_OK.to_string(),
                elapsed_ms: success.elapsed_ms,
                http_status: Some(success.status.as_u16()),
                provider: provider.to_string(),
                model: model.to_string(),
                reply_preview: Some(truncate_chars(reply.trim(), 240)),
                raw_error: String::new(),
            },
            None => ConnectivityReport {
                ok: false,
                reason: REASON_EMPTY_REPLY.to_string(),
                elapsed_ms: success.elapsed_ms,
                http_status: Some(success.status.as_u16()),
                provider: provider.to_string(),
                model: model.to_string(),
                reply_preview: None,
                raw_error: redact_secrets(&truncate_chars(&success.body, 1200)),
            },
        },
        Err(failure) => {
            let reason = match &failure {
                ExchangeFailure::BodyTimeout { .. } => REASON_TIMEOUT_AFTER_HEADERS,
                ExchangeFailure::NoResponse { .. } => REASON_TIMEOUT,
                ExchangeFailure::HttpStatus { .. } => REASON_HTTP_ERROR,
                ExchangeFailure::Transport { .. } | ExchangeFailure::BodyRead { .. } => {
                    REASON_NETWORK
                }
            };
            ConnectivityReport {
                ok: false,
                reason: reason.to_string(),
                elapsed_ms: failure.elapsed_ms(),
                http_status: failure.status().map(|status| status.as_u16()),
                provider: provider.to_string(),
                model: model.to_string(),
                reply_preview: None,
                raw_error: failure.raw().to_string(),
            }
        }
    }
}

fn blank_report(provider: &str, model: &str, reason: &str, raw: &str) -> ConnectivityReport {
    ConnectivityReport {
        ok: false,
        reason: reason.to_string(),
        elapsed_ms: 0,
        http_status: None,
        provider: provider.to_string(),
        model: model.to_string(),
        reply_preview: None,
        raw_error: raw.to_string(),
    }
}

fn nonempty(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn extract_openai_reply(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    nonempty(
        value
            .pointer("/choices/0/message/content")
            .and_then(|item| item.as_str())
            .unwrap_or(""),
    )
}

fn extract_anthropic_reply(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let blocks = value.get("content")?.as_array()?;
    let text = blocks.iter().find_map(|block| {
        if block.get("type").and_then(|item| item.as_str()) == Some("text") {
            block.get("text").and_then(|item| item.as_str())
        } else {
            None
        }
    })?;
    nonempty(text)
}

fn extract_gemini_reply(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let text = value
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(|item| item.as_str())?;
    nonempty(text)
}

fn extract_ollama_reply(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let text = value.get("response")?.as_str()?;
    nonempty(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::provider_http::ExchangeFailure;
    use reqwest::StatusCode;

    #[test]
    fn body_stall_is_timeout_after_headers() {
        let report = into_report(
            "deepseek",
            "deepseek-v4-flash",
            Err(ExchangeFailure::BodyTimeout {
                status: StatusCode::OK,
                elapsed_ms: 20_000,
                raw: "DeepSeek chat completions: HTTP 200 OK received in 300ms, but the response body did not finish within 20s".to_string(),
            }),
            extract_openai_reply,
        );
        assert!(!report.ok);
        assert_eq!(report.reason, REASON_TIMEOUT_AFTER_HEADERS);
        assert_eq!(report.http_status, Some(200));
        assert!(report.raw_error.contains("response body did not finish"));
    }

    #[test]
    fn openai_reply_counts_as_success() {
        let report = into_report(
            "openai",
            "gpt-5-mini",
            Ok(ExchangeSuccess {
                status: StatusCode::OK,
                body: r#"{"choices":[{"message":{"content":"{\"reply\":\"pong\"}"}}]}"#.to_string(),
                elapsed_ms: 800,
            }),
            extract_openai_reply,
        );
        assert!(report.ok);
        assert_eq!(report.reply_preview.as_deref(), Some("{\"reply\":\"pong\"}"));
        assert!(report.raw_error.is_empty());
    }

    #[test]
    fn empty_content_keeps_the_raw_body() {
        let report = into_report(
            "deepseek",
            "deepseek-v4-flash",
            Ok(ExchangeSuccess {
                status: StatusCode::OK,
                body: r#"{"choices":[{"message":{"content":"","reasoning_content":"thinking"}}]}"#
                    .to_string(),
                elapsed_ms: 1200,
            }),
            extract_openai_reply,
        );
        assert_eq!(report.reason, REASON_EMPTY_REPLY);
        assert!(report.raw_error.contains("reasoning_content"));
    }
}
