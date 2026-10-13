use crate::constants::providers;
use crate::errors::AppError;
use crate::models::{
    AnthropicConfig, AppSettings, DeepSeekConfig, GeminiConfig, OllamaConfig, OpenAiConfig,
};
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct ProviderSettingsService;

impl ProviderSettingsService {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_provider_config(
        &self,
        provider: &str,
        config: Value,
        settings: &mut AppSettings,
    ) -> Result<(), AppError> {
        match provider {
            providers::OPENAI => {
                let parsed: OpenAiConfig = Self::parse_config(config, "OpenAI")?;
                info!("OpenAI configuration updated");
                settings.open_ai_config = Some(parsed);
            }
            providers::DEEPSEEK => {
                let parsed: DeepSeekConfig = Self::parse_config(config, "DeepSeek")?;
                info!("DeepSeek configuration updated");
                settings.deepseek_config = Some(parsed);
            }
            providers::ANTHROPIC => {
                let parsed: AnthropicConfig = Self::parse_config(config, "Anthropic")?;
                info!("Anthropic configuration updated");
                settings.anthropic_config = Some(parsed);
            }
            providers::GEMINI => {
                let parsed: GeminiConfig = Self::parse_config(config, "Gemini")?;
                info!("Gemini configuration updated");
                settings.gemini_config = Some(parsed);
            }
            providers::OLLAMA => {
                let parsed: OllamaConfig = Self::parse_config(config, "Ollama")?;
                info!("Ollama configuration updated");
                settings.ollama_config = Some(parsed);
            }
            _ => {
                warn!("Attempted to update unknown provider: {}", provider);
                return Err(AppError::provider_not_supported(provider));
            }
        }

        debug!("Provider switched to {}", provider);
        settings.ai_provider = provider.to_string();
        Ok(())
    }

    pub fn extract_ollama_endpoint(&self, settings: &AppSettings) -> String {
        let raw = settings
            .ollama_config
            .as_ref()
            .map(|cfg| cfg.endpoint.as_str());
        crate::services::ollama::normalize_ollama_endpoint(raw)
    }

    fn parse_config<T: DeserializeOwned>(value: Value, label: &str) -> Result<T, AppError> {
        serde_json::from_value(value)
            .map_err(|e| AppError::validation(format!("Invalid {} config: {}", label, e)))
    }
}

impl Default for ProviderSettingsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::providers;

    fn blank_settings() -> AppSettings {
        AppSettings {
            ai_provider: String::new(),
            open_ai_config: None,
            deepseek_config: None,
            anthropic_config: None,
            gemini_config: None,
            ollama_config: None,
            theme: "light".to_string(),
            export_settings: None,
            explanation_language: Some("English".into()),
            ui_language: None,
            enable_global_lookup: true,
            typography_mode: "classic".into(),
            technical_query: false,
            technical_domain: crate::models::default_technical_domain(),
            clipboard_click_search: true,
            vocab_note_enabled: false,
            vocab_note_path: None,
            vocab_note_split_files: false,
            dev_log_enabled: false,
            max_meanings_display: 5,
            voice: crate::models::VoiceSettings::default(),
        }
    }

    #[test]
    fn applies_openai_config_and_sets_provider() {
        let service = ProviderSettingsService::new();
        let mut settings = blank_settings();
        let payload = serde_json::json!({
            "apiKey": "test-key",
            "model": "gpt-test"
        });

        service
            .apply_provider_config(providers::OPENAI, payload, &mut settings)
            .unwrap();

        assert_eq!(settings.ai_provider, providers::OPENAI);
        assert!(settings.open_ai_config.is_some());
        assert_eq!(settings.open_ai_config.unwrap().model, "gpt-test");
    }

    #[test]
    fn migrates_openai_deepseek_alias() {
        let mut settings = blank_settings();
        settings.ai_provider = providers::OPENAI.to_string();
        settings.open_ai_config = Some(OpenAiConfig {
            api_key: "sk-ds".into(),
            model: "deepseek-v4-flash".into(),
        });
        assert!(settings.migrate_misfiled_deepseek());
        assert_eq!(settings.ai_provider, providers::DEEPSEEK);
        assert!(settings.open_ai_config.is_none());
        assert_eq!(
            settings.deepseek_config.as_ref().unwrap().model,
            "deepseek-v4-flash"
        );
    }

    #[test]
    fn rejects_unknown_provider() {
        let service = ProviderSettingsService::new();
        let mut settings = blank_settings();
        let err = service
            .apply_provider_config("made-up", serde_json::json!({}), &mut settings)
            .unwrap_err();
        assert!(matches!(err, AppError::ProviderNotSupported(_)));
    }

    #[test]
    fn extract_ollama_endpoint_defaults_when_missing() {
        let service = ProviderSettingsService::new();
        let endpoint = service.extract_ollama_endpoint(&blank_settings());
        assert_eq!(endpoint, "http://localhost:11434");
    }

    #[test]
    fn extract_ollama_endpoint_uses_configured_value() {
        let service = ProviderSettingsService::new();
        let mut settings = blank_settings();
        settings.ollama_config = Some(OllamaConfig {
            endpoint: "http://custom:9999/".into(),
            model: "phi3".into(),
        });

        let endpoint = service.extract_ollama_endpoint(&settings);
        assert_eq!(endpoint, "http://custom:9999");
    }

    #[test]
    fn invalid_payload_surfaces_validation_error() {
        let service = ProviderSettingsService::new();
        let mut settings = blank_settings();
        let err = service
            .apply_provider_config(
                providers::OLLAMA,
                serde_json::json!({ "model": 42 }),
                &mut settings,
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
