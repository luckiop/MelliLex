use std::future::Future;
use std::time::Duration;

use backoff::backoff::Backoff;
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use log::warn;
use reqwest::{Client, Error as ReqwestError, Response, StatusCode};
use tokio::time::sleep;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryIntent {
    Idempotent,
    NonIdempotent,
}

#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_interval: Duration,
    pub max_interval: Duration,
    pub multiplier: f64,
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3, // initial try + 2 retries
            initial_interval: Duration::from_millis(250),
            max_interval: Duration::from_secs(2),
            multiplier: 2.0,
            jitter: 0.25,
        }
    }
}

impl RetryConfig {
    fn build_strategy(&self) -> ExponentialBackoff {
        ExponentialBackoffBuilder::new()
            .with_initial_interval(self.initial_interval)
            .with_multiplier(self.multiplier)
            .with_max_interval(self.max_interval)
            .with_randomization_factor(self.jitter)
            .with_max_elapsed_time(None)
            .build()
    }
}

#[derive(Clone)]
pub struct RetriableClient {
    client: Client,
    config: RetryConfig,
}

impl RetriableClient {
    pub fn new(client: Client, config: RetryConfig) -> Self {
        Self { client, config }
    }

    pub async fn send_with_retry<F, Fut>(
        &self,
        label: &str,
        intent: RetryIntent,
        mut operation: F,
    ) -> Result<Response, ReqwestError>
    where
        F: FnMut(&Client) -> Fut,
        Fut: Future<Output = Result<Response, ReqwestError>>,
    {
        let mut attempts = 0_u32;
        let mut strategy = self.config.build_strategy();
        let max_attempts = match intent {
            RetryIntent::Idempotent => self.config.max_attempts,
            // Chat completions must not retry: lookups already fan out 3 requests.
            RetryIntent::NonIdempotent => 1,
        };

        loop {
            attempts += 1;
            log::debug!("[RetriableClient] {} attempt {}/{}", label, attempts, max_attempts);
            let result = operation(&self.client).await;
            match result {
                Ok(response) => {
                    if !self.should_retry_status(response.status(), intent)
                        || attempts >= max_attempts
                    {
                        return Ok(response);
                    }

                    if let Some(delay) = strategy.next_backoff() {
                        warn!(
                            "[RetriableClient] {} attempt {} hit {} - retrying in {:?}",
                            label,
                            attempts,
                            response.status(),
                            delay
                        );
                        sleep(delay).await;
                        continue;
                    } else {
                        return Ok(response);
                    }
                }
                Err(err) => {
                    if attempts >= max_attempts || !self.should_retry_error(&err, intent) {
                        return Err(err);
                    }

                    if let Some(delay) = strategy.next_backoff() {
                        warn!(
                            "[RetriableClient] {} attempt {} failed: {} - retrying in {:?}",
                            label, attempts, err, delay
                        );
                        sleep(delay).await;
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }

    fn should_retry_status(&self, status: StatusCode, intent: RetryIntent) -> bool {
        if status == StatusCode::TOO_MANY_REQUESTS
            || status == StatusCode::REQUEST_TIMEOUT
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::SERVICE_UNAVAILABLE
            || status == StatusCode::GATEWAY_TIMEOUT
            || status.is_server_error()
        {
            return true;
        }

        matches!(intent, RetryIntent::Idempotent) && status == StatusCode::CONFLICT
    }

    fn should_retry_error(&self, err: &ReqwestError, intent: RetryIntent) -> bool {
        if err.is_timeout() || err.is_connect() {
            return true;
        }

        matches!(intent, RetryIntent::Idempotent) && err.is_request()
    }
}

impl Default for RetriableClient {
    fn default() -> Self {
        Self::new(Client::new(), RetryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_config_builds_strategy() {
        let cfg = RetryConfig::default();
        let mut strat = cfg.build_strategy();
        assert!(strat.next_backoff().is_some());
    }

    #[test]
    fn retries_server_errors() {
        let client = RetriableClient::default();
        assert!(client.should_retry_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            RetryIntent::NonIdempotent
        ));
        assert!(!client.should_retry_status(StatusCode::BAD_REQUEST, RetryIntent::NonIdempotent));
    }
}
