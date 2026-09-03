use crate::errors::AppError;
use crate::models::AppSettings;
use crate::services::ai_provider::{AiModel, PromptSender};
use crate::services::ollama::OllamaService;
use crate::services::provider_registry::ProviderRegistry;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

type CachedEntry = (u64, String, Arc<dyn PromptSender>);

pub struct ProviderResolver {
    provider_registry: ProviderRegistry,
    ollama_service: OllamaService,
    cached: RwLock<Option<CachedEntry>>,
}

impl ProviderResolver {
    pub fn new(provider_registry: ProviderRegistry) -> Self {
        Self {
            provider_registry,
            ollama_service: OllamaService::default(),
            cached: RwLock::new(None),
        }
    }

    pub fn resolve_prompt_sender(
        &self,
        settings: &AppSettings,
    ) -> Result<(Arc<dyn PromptSender>, String), AppError> {
        let fingerprint = Self::settings_fingerprint(settings);

        {
            let guard = self.cached.read().expect("provider cache lock poisoned");
            if let Some((fp, name, sender)) = guard.as_ref() {
                if *fp == fingerprint {
                    return Ok((sender.clone(), name.clone()));
                }
            }
        }

        let (provider, name) = self.provider_registry.resolve_current(settings)?;
        {
            let mut guard = self.cached.write().expect("provider cache lock poisoned");
            *guard = Some((fingerprint, name.clone(), provider.clone()));
        }
        Ok((provider, name))
    }

    pub fn invalidate_cache(&self) {
        let mut guard = self.cached.write().expect("provider cache lock poisoned");
        *guard = None;
    }

    pub fn ollama_service(&self) -> &OllamaService {
        &self.ollama_service
    }

    pub async fn fetch_models(
        &self,
        provider: &str,
        credential: &str,
    ) -> Result<Vec<AiModel>, AppError> {
        self.provider_registry.fetch_models(provider, credential).await
    }

    pub async fn test_api_key(
        &self,
        provider: &str,
        credential: &str,
    ) -> Result<bool, AppError> {
        self.provider_registry.test_credential(provider, credential).await
    }

    fn settings_fingerprint(settings: &AppSettings) -> u64 {
        let mut hasher = DefaultHasher::new();
        settings.ai_provider.hash(&mut hasher);
        settings.theme.hash(&mut hasher);

        if let Some(cfg) = &settings.open_ai_config {
            true.hash(&mut hasher);
            cfg.api_key.hash(&mut hasher);
            cfg.model.hash(&mut hasher);
        } else {
            false.hash(&mut hasher);
        }

        if let Some(cfg) = &settings.deepseek_config {
            true.hash(&mut hasher);
            cfg.api_key.hash(&mut hasher);
            cfg.model.hash(&mut hasher);
        } else {
            false.hash(&mut hasher);
        }

        if let Some(cfg) = &settings.anthropic_config {
            true.hash(&mut hasher);
            cfg.api_key.hash(&mut hasher);
            cfg.model.hash(&mut hasher);
        } else {
            false.hash(&mut hasher);
        }

        if let Some(cfg) = &settings.gemini_config {
            true.hash(&mut hasher);
            cfg.api_key.hash(&mut hasher);
            cfg.model.hash(&mut hasher);
        } else {
            false.hash(&mut hasher);
        }

        if let Some(cfg) = &settings.ollama_config {
            true.hash(&mut hasher);
            cfg.endpoint.hash(&mut hasher);
            cfg.model.hash(&mut hasher);
        } else {
            false.hash(&mut hasher);
        }

        hasher.finish()
    }
}
