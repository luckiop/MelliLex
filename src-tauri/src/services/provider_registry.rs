use crate::constants::providers;
use crate::errors::AppError;
use crate::models::AppSettings;
use crate::services::ai_provider::{AiModel, PromptSender};
use crate::services::anthropic::AnthropicService;
use crate::services::gemini::GeminiService;
use crate::services::ollama::{normalize_ollama_endpoint, OllamaService};
use crate::services::openai::OpenAIService;
use std::sync::Arc;

pub struct ProviderRegistry;

impl ProviderRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_current(
        &self,
        settings: &AppSettings,
    ) -> Result<(Arc<dyn PromptSender>, String), AppError> {
        let key = settings.ai_provider.as_str();
        let provider: Arc<dyn PromptSender> = match key {
            providers::OPENAI => build_openai_provider(settings)?,
            providers::ANTHROPIC => build_anthropic_provider(settings)?,
            providers::GEMINI => build_gemini_provider(settings)?,
            providers::OLLAMA => build_ollama_provider(settings)?,
            _ => return Err(AppError::provider_not_supported(key)),
        };
        Ok((provider, key.to_string()))
    }

    pub async fn fetch_models(
        &self,
        provider: &str,
        credential: &str,
    ) -> Result<Vec<AiModel>, AppError> {
        match provider {
            providers::OPENAI => OpenAIService::fetch_models_internal(credential).await,
            providers::ANTHROPIC => AnthropicService::fetch_models_internal(credential).await,
            providers::GEMINI => GeminiService::fetch_models_internal(credential).await,
            providers::OLLAMA => {
                let endpoint = normalize_ollama_endpoint(Some(credential));
                OllamaService::fetch_models_internal(&endpoint).await
            }
            _ => return Err(AppError::provider_not_supported(provider)),
        }
        .map_err(AppError::from)
    }

    pub async fn test_credential(
        &self,
        provider: &str,
        credential: &str,
    ) -> Result<bool, AppError> {
        match provider {
            providers::OPENAI => OpenAIService::test_api_key_internal(credential).await,
            providers::ANTHROPIC => AnthropicService::test_api_key_internal(credential).await,
            providers::GEMINI => GeminiService::test_api_key_internal(credential).await,
            providers::OLLAMA => {
                let endpoint = normalize_ollama_endpoint(Some(credential));
                OllamaService::test_endpoint_internal(&endpoint).await
            }
            _ => return Err(AppError::provider_not_supported(provider)),
        }
        .map_err(AppError::from)
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn build_openai_provider(settings: &AppSettings) -> Result<Arc<dyn PromptSender>, AppError> {
    let config = settings
        .open_ai_config
        .as_ref()
        .ok_or_else(|| AppError::provider_not_configured(providers::OPENAI))?;

    log::info!("Creating OpenAI provider with model: {}", config.model);
    Ok(Arc::new(OpenAIService::new(
        config.api_key.clone(),
        config.model.clone(),
    )))
}

fn build_anthropic_provider(settings: &AppSettings) -> Result<Arc<dyn PromptSender>, AppError> {
    let config = settings
        .anthropic_config
        .as_ref()
        .ok_or_else(|| AppError::provider_not_configured(providers::ANTHROPIC))?;

    log::info!("Creating Anthropic provider with model: {}", config.model);
    Ok(Arc::new(AnthropicService::new(
        config.api_key.clone(),
        config.model.clone(),
    )))
}

fn build_gemini_provider(settings: &AppSettings) -> Result<Arc<dyn PromptSender>, AppError> {
    let config = settings
        .gemini_config
        .as_ref()
        .ok_or_else(|| AppError::provider_not_configured(providers::GEMINI))?;

    log::info!("Creating Gemini provider with model: {}", config.model);
    Ok(Arc::new(GeminiService::new(
        config.api_key.clone(),
        config.model.clone(),
    )))
}

fn build_ollama_provider(settings: &AppSettings) -> Result<Arc<dyn PromptSender>, AppError> {
    let config = settings
        .ollama_config
        .as_ref()
        .ok_or_else(|| AppError::provider_not_configured(providers::OLLAMA))?;

    let endpoint = if config.endpoint.is_empty() {
        "http://localhost:11434".to_string()
    } else {
        config.endpoint.clone()
    };
    let model = config.model.clone();

    if model.is_empty() {
        return Err(AppError::provider_not_configured(providers::OLLAMA));
    }

    log::info!(
        "Creating Ollama provider with model: {} at {}",
        model,
        endpoint
    );
    Ok(Arc::new(OllamaService::new(endpoint, model)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AnthropicConfig, OpenAiConfig};

    fn openai_settings() -> AppSettings {
        AppSettings {
            ai_provider: providers::OPENAI.to_string(),
            open_ai_config: Some(OpenAiConfig {
                api_key: "test".into(),
                model: "gpt".into(),
            }),
            anthropic_config: None,
            gemini_config: None,
            ollama_config: None,
            theme: "light".into(),
            export_settings: None,
            explanation_language: Some("English".into()),
            ui_language: None,
            enable_global_lookup: true,
            typography_mode: "classic".into(),
            technical_query: false,
            vocab_note_enabled: false,
            vocab_note_path: None,
            vocab_note_split_files: false,
            dev_log_enabled: false,
        }
    }

    fn anthropic_settings() -> AppSettings {
        AppSettings {
            ai_provider: providers::ANTHROPIC.to_string(),
            open_ai_config: None,
            anthropic_config: Some(AnthropicConfig {
                api_key: "test".into(),
                model: "claude".into(),
            }),
            gemini_config: None,
            ollama_config: None,
            theme: "light".into(),
            export_settings: None,
            explanation_language: Some("English".into()),
            ui_language: None,
            enable_global_lookup: true,
            typography_mode: "classic".into(),
            technical_query: false,
            vocab_note_enabled: false,
            vocab_note_path: None,
            vocab_note_split_files: false,
            dev_log_enabled: false,
        }
    }

    #[test]
    fn resolves_openai_when_configured() {
        let registry = ProviderRegistry::default();
        let (_, key) = registry.resolve_current(&openai_settings()).unwrap();
        assert_eq!(key, providers::OPENAI);
    }

    #[test]
    fn resolves_anthropic_when_configured() {
        let registry = ProviderRegistry::default();
        let (_, key) = registry.resolve_current(&anthropic_settings()).unwrap();
        assert_eq!(key, providers::ANTHROPIC);
    }

    #[test]
    fn errors_when_provider_unregistered() {
        let mut settings = openai_settings();
        settings.ai_provider = "unknown".into();
        let registry = ProviderRegistry::default();
        let result = registry.resolve_current(&settings);
        assert!(matches!(result, Err(AppError::ProviderNotSupported(_))));
    }

    #[test]
    fn errors_when_config_missing() {
        let mut settings = openai_settings();
        settings.open_ai_config = None;
        let registry = ProviderRegistry::default();
        let result = registry.resolve_current(&settings);
        assert!(matches!(result, Err(AppError::ProviderNotConfigured(_))));
    }
}
