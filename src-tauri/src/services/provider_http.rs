use std::future::Future;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use log::error;
use reqwest::{Response, StatusCode};
use tokio::time::timeout;

/// How long a lookup may wait for response headers.
pub const LOOKUP_HEADER_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a lookup may wait for the body after headers.
/// Shorter than the UI safety timer so the real provider error wins.
pub const LOOKUP_BODY_TIMEOUT: Duration = Duration::from_secs(75);
/// Connectivity check: fail fast when the model accepts the request and then stalls.
pub const PROBE_HEADER_TIMEOUT: Duration = Duration::from_secs(15);
pub const PROBE_BODY_TIMEOUT: Duration = Duration::from_secs(20);

pub struct ExchangeSuccess {
    pub status: StatusCode,
    pub body: String,
    pub elapsed_ms: u64,
}

#[derive(Debug)]
pub enum ExchangeFailure {
    NoResponse { elapsed_ms: u64, raw: String },
    Transport { elapsed_ms: u64, raw: String },
    HttpStatus { status: StatusCode, elapsed_ms: u64, raw: String },
    BodyTimeout { status: StatusCode, elapsed_ms: u64, raw: String },
    BodyRead { status: StatusCode, elapsed_ms: u64, raw: String },
}

impl ExchangeFailure {
    pub fn elapsed_ms(&self) -> u64 {
        match self {
            Self::NoResponse { elapsed_ms, .. }
            | Self::Transport { elapsed_ms, .. }
            | Self::HttpStatus { elapsed_ms, .. }
            | Self::BodyTimeout { elapsed_ms, .. }
            | Self::BodyRead { elapsed_ms, .. } => *elapsed_ms,
        }
    }

    pub fn raw(&self) -> &str {
        match self {
            Self::NoResponse { raw, .. }
            | Self::Transport { raw, .. }
            | Self::HttpStatus { raw, .. }
            | Self::BodyTimeout { raw, .. }
            | Self::BodyRead { raw, .. } => raw,
        }
    }

    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::HttpStatus { status, .. }
            | Self::BodyTimeout { status, .. }
            | Self::BodyRead { status, .. } => Some(*status),
            Self::NoResponse { .. } | Self::Transport { .. } => None,
        }
    }

    pub fn into_anyhow(self) -> anyhow::Error {
        anyhow::anyhow!("{}", self.raw())
    }
}

pub async fn ensure_success(label: &str, response: Response) -> Result<Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    error!(
        "{} API error: status {}, body: {}",
        label,
        status,
        redact_secrets(&body)
    );
    let detail = extract_error_message(&body).unwrap_or_else(|| status.to_string());
    Err(anyhow!("{}: {}", label, detail))
}

/// Wait for headers, then for the full body.
///
/// `reqwest`'s `send()` finishes when headers arrive. A provider can return
/// HTTP 200 immediately and then never close the body; that is what makes
/// lookups sit until the UI timeout.
pub async fn exchange<F>(
    label: &str,
    header_timeout: Duration,
    body_timeout: Duration,
    send: F,
) -> Result<ExchangeSuccess, ExchangeFailure>
where
    F: Future<Output = Result<Response, reqwest::Error>>,
{
    let started = Instant::now();
    let response = match timeout(header_timeout, send).await {
        Ok(Ok(response)) => response,
        Ok(Err(err)) => {
            let raw = redact_secrets(&format!("{label}: {err}"));
            error!("{raw}");
            return Err(ExchangeFailure::Transport {
                elapsed_ms: elapsed_ms(started),
                raw,
            });
        }
        Err(_) => {
            let raw = format!(
                "{label}: no HTTP response within {}s",
                header_timeout.as_secs()
            );
            error!("{raw}");
            return Err(ExchangeFailure::NoResponse {
                elapsed_ms: elapsed_ms(started),
                raw,
            });
        }
    };

    let status = response.status();
    let header_ms = started.elapsed().as_millis();
    if !status.is_success() {
        let body = match timeout(Duration::from_secs(15), response.text()).await {
            Ok(Ok(body)) => body,
            Ok(Err(err)) => err.to_string(),
            Err(_) => format!("error body did not finish within 15s"),
        };
        let detail = extract_error_message(&body).unwrap_or_else(|| status.to_string());
        let raw = redact_secrets(&format!(
            "{label}: HTTP {status}: {detail}; body: {}",
            truncate_chars(&body, 800)
        ));
        error!("{raw}");
        return Err(ExchangeFailure::HttpStatus {
            status,
            elapsed_ms: elapsed_ms(started),
            raw,
        });
    }

    match timeout(body_timeout, response.text()).await {
        Ok(Ok(body)) => Ok(ExchangeSuccess {
            status,
            body,
            elapsed_ms: elapsed_ms(started),
        }),
        Ok(Err(err)) => {
            let raw = redact_secrets(&format!("{label}: HTTP {status}: {err}"));
            error!("{raw}");
            Err(ExchangeFailure::BodyRead {
                status,
                elapsed_ms: elapsed_ms(started),
                raw,
            })
        }
        Err(_) => {
            let raw = format!(
                "{label}: HTTP {status} received in {header_ms}ms, but the response body did not finish within {}s",
                body_timeout.as_secs()
            );
            error!("{raw}");
            Err(ExchangeFailure::BodyTimeout {
                status,
                elapsed_ms: elapsed_ms(started),
                raw,
            })
        }
    }
}

/// Best-effort extraction of a human-readable message from common provider
/// error payloads (OpenAI / Anthropic / Gemini all use `{"error":{"message":...}}`).
fn extract_error_message(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let err = value.get("error")?;
    if let Some(msg) = err.get("message").and_then(|v| v.as_str()) {
        return Some(msg.to_string());
    }
    err.as_str().map(|s| s.to_string())
}

pub fn log_status_if_error(label: &str, status: StatusCode) {
    if !status.is_success() {
        error!("{} API error: status {}", label, status);
    }
}

pub fn redact_secrets(message: &str) -> String {
    let mut result = message.to_string();
    for marker in ["api_key=", "apiKey=", "key="] {
        let mut search_from = 0;
        loop {
            let Some(relative) = result[search_from..].find(marker) else {
                break;
            };
            let start = search_from + relative;
            let value_start = start + marker.len();
            let rest = &result[value_start..];
            let value_end = rest
                .find(|c: char| c == '&' || c == ' ' || c == '"' || c == '\'' || c == ')')
                .unwrap_or(rest.len());
            if value_end == 0 {
                search_from = value_start;
                continue;
            }
            result.replace_range(value_start..value_start + value_end, "REDACTED");
            search_from = value_start + "REDACTED".len();
        }
    }
    if let Some(start) = result.find("Bearer ") {
        let value_start = start + "Bearer ".len();
        let rest = &result[value_start..];
        let value_end = rest
            .find(|c: char| c == ' ' || c == '"' || c == '\'' || c == ')')
            .unwrap_or(rest.len());
        if value_end > 0 {
            result.replace_range(value_start..value_start + value_end, "REDACTED");
        }
    }
    result
}

pub fn truncate_chars(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push('…');
    truncated
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_query_keys_and_bearer_tokens() {
        let raw = "GET /v1?key=sk-secret&x=1 Bearer sk-live failed";
        let cleaned = redact_secrets(raw);
        assert!(!cleaned.contains("sk-secret"));
        assert!(!cleaned.contains("sk-live"));
        assert!(cleaned.contains("key=REDACTED"));
        assert!(cleaned.contains("Bearer REDACTED"));
    }

    #[test]
    fn truncate_keeps_char_boundaries() {
        let text = "指".repeat(3);
        assert_eq!(truncate_chars(&text, 2).chars().count(), 3);
    }
}
