use crate::constants::{explore_features, providers};
use crate::db::Database;
use crate::errors::AppError;
use crate::models::{
    AppSettings, CachedExploreFeatures, CachedFormalityData, DomainExploration,
    FormalityAlternative, MistakeItem, PracticeExercise, SpellCheckResponse, UsagePattern,
    ZhEnResult,
};
use crate::services::ai_provider::AiModel;
use crate::services::dictionary_service::DictionaryService;
use crate::services::explore_cache_repository::ExploreCacheRepository;
use crate::services::explore_feature_coordinator::ExploreFeatureCoordinator;
use crate::services::exploration_service::ExplorationService;
use crate::services::orchestration::{
    AppHandleEmitter, PhraseAppHandleEmitter, PhraseProgressiveEmitter,
    PhraseSectionGenerator, ProgressiveEmitter, ProviderResolver, SectionGenerator,
    SpellCheckCoordinator,
};
use crate::services::phrase_detection::PhraseDetector;
use crate::services::phrase_service::PhraseService;
use crate::services::provider_registry::ProviderRegistry;
use crate::services::settings_service::ProviderSettingsService;
use crate::validation;
use log::info;
use std::sync::Arc;
use tauri::AppHandle;

pub struct CommandOrchestrator {
    provider_resolver: ProviderResolver,
    settings_service: ProviderSettingsService,
    spellcheck_coordinator: SpellCheckCoordinator,
}

impl CommandOrchestrator {
    pub fn new(provider_registry: ProviderRegistry) -> Self {
        Self {
            provider_resolver: ProviderResolver::new(provider_registry),
            settings_service: ProviderSettingsService::new(),
            spellcheck_coordinator: SpellCheckCoordinator::new(),
        }
    }

    fn load_settings(&self, db: &Database) -> Result<AppSettings, AppError> {
        db.get_settings()
    }

    fn create_dictionary_service(
        &self,
        settings: &AppSettings,
    ) -> Result<(DictionaryService, String), AppError> {
        let (provider, name) = self.provider_resolver.resolve_prompt_sender(settings)?;
        Ok((DictionaryService::new(provider), name))
    }

    fn create_exploration_service(
        &self,
        settings: &AppSettings,
    ) -> Result<(ExplorationService, String), AppError> {
        let (provider, name) = self.provider_resolver.resolve_prompt_sender(settings)?;
        Ok((ExplorationService::new(provider), name))
    }

    fn create_phrase_service(
        &self,
        settings: &AppSettings,
    ) -> Result<(PhraseService, String), AppError> {
        let (provider, name) = self.provider_resolver.resolve_prompt_sender(settings)?;
        Ok((PhraseService::new(provider), name))
    }

    fn build_explore_coordinator<'a>(
        &'a self,
        db: &'a Database,
        exploration_service: &'a crate::services::exploration_service::ExplorationService,
        provider_name: String,
        language: String,
    ) -> ExploreFeatureCoordinator<'a> {
        let scope = provider_cache_key(&provider_name, &language);
        ExploreFeatureCoordinator::new(
            ExploreCacheRepository::new(db),
            scope,
            provider_name,
            language,
            exploration_service,
            db,
        )
    }

    fn resolve_language(settings: &AppSettings) -> String {
        settings
            .explanation_language
            .as_deref()
            .unwrap_or("English")
            .to_string()
    }

    fn build_explore_coordinator_for_settings<'a>(
        &'a self,
        db: &'a Database,
        exploration_service: &'a crate::services::exploration_service::ExplorationService,
        provider_name: String,
        settings: &AppSettings,
    ) -> (ExploreFeatureCoordinator<'a>, String) {
        let language = Self::resolve_language(settings);
        let coordinator =
            self.build_explore_coordinator(db, exploration_service, provider_name, language.clone());
        (coordinator, language)
    }

    fn build_provider_priority_list(settings: &AppSettings) -> Vec<String> {
        let mut priorities = Vec::with_capacity(6);
        priorities.push(settings.ai_provider.clone());

        const FALLBACK_PROVIDERS: [&str; 5] = [
            providers::DEEPSEEK,
            providers::ANTHROPIC,
            providers::OPENAI,
            providers::GEMINI,
            providers::OLLAMA,
        ];

        for provider in FALLBACK_PROVIDERS {
            if !priorities.iter().any(|p| p == provider) {
                priorities.push(provider.to_string());
            }
        }

        priorities
    }

    fn try_emit_cached_progressive<E: ProgressiveEmitter + ?Sized>(
        &self,
        db: &Database,
        word: &str,
        emitter: &E,
        providers: &[String],
    ) -> Result<bool, AppError> {
        for provider in providers {
            if let Some(cached) = db.get_cached_word_progressive(word, provider)? {
                info!(
                    "Found cached word progressive data for '{}' from provider '{}'",
                    word, provider
                );
                emitter.emit_section1(&cached.section1)?;
                emitter.emit_section2(&cached.section2)?;
                emitter.emit_section3(&cached.section3)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn try_emit_cached_phrase<E: PhraseProgressiveEmitter + ?Sized>(
        db: &Database,
        phrase: &str,
        emitter: &E,
        providers: &[String],
    ) -> Result<bool, AppError> {
        for provider in providers {
            if let Some(cached) = db.get_cached_phrase(phrase, provider)? {
                info!(
                    "Found cached phrase data for '{}' from provider '{}'",
                    phrase, provider
                );
                emitter.emit_section1(&cached.section1)?;
                emitter.emit_section2(&cached.section2)?;
                emitter.emit_section3(&cached.section3)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn check_spelling(&self, word: &str) -> Result<SpellCheckResponse, AppError> {
        self.spellcheck_coordinator.check_spelling(word).await
    }

    pub async fn get_word_variations(&self, word: &str) -> Result<Vec<String>, AppError> {
        self.spellcheck_coordinator.get_word_variations(word).await
    }

    pub async fn search_word_progressive(
        &self,
        word: &str,
        app: &AppHandle,
        db: &Database,
    ) -> Result<(), AppError> {
        let emitter = AppHandleEmitter::new(app);
        self.search_word_progressive_with_emitter(word, db, &emitter)
            .await
    }

    pub async fn search_word_progressive_with_emitter<E: ProgressiveEmitter + ?Sized>(
        &self,
        word: &str,
        db: &Database,
        emitter: &E,
    ) -> Result<(), AppError> {
        let validated_word = validation::validate_word_query(word)?;
        info!("Progressive word search for: {}", validated_word);

        let settings = self.load_settings(db)?;

        let language = Self::resolve_language(&settings);
        info!("Using explanation language: {}", language);
        let technical_query = settings.technical_query;
        if technical_query {
            info!("Technical query enabled: CS/control/robotics sense first");
        }

        let provider_priorities = Self::build_provider_priority_list(&settings);
        // Technical lookups must not reuse a general cached definition.
        if !technical_query
            && self.try_emit_cached_progressive(
                db,
                &validated_word,
                emitter,
                &provider_priorities,
            )?
        {
            return Ok(());
        }

        let (dictionary_service, provider_name) = self.create_dictionary_service(&settings)?;

        info!(
            "Launching progressive generation for '{}' (emitting section 1 immediately)",
            validated_word
        );

        let dictionary_service = Arc::new(dictionary_service);
        let combined = SectionGenerator::generate_progressive(
            dictionary_service,
            &validated_word,
            &language,
            technical_query,
            emitter,
        )
        .await?;

        if !technical_query {
            db.cache_word_progressive(word, &combined, &provider_name)?;
        }
        db.add_to_history(word, &provider_name)?;

        info!("Completed progressive word search for: {}", word);

        Ok(())
    }

    pub async fn search_zh_en(&self, query: &str, db: &Database) -> Result<ZhEnResult, AppError> {
        let validated = validation::validate_word_query(query)?;
        info!("Zh-en search for: {}", validated);

        let settings = self.load_settings(db)?;
        let (dictionary_service, provider_name) = self.create_dictionary_service(&settings)?;
        let cache_scope = format!("{}:zhen", provider_name);

        if let Some(cached) = db.get_cached_zh_en(&validated, &cache_scope)? {
            info!("Zh-en cache hit for '{}'", validated);
            return Ok(cached);
        }

        let (mut result, _) = dictionary_service.generate_zh_en(&validated).await?;
        if result.query.trim().is_empty() {
            result.query = validated.clone();
        }
        db.cache_zh_en(&validated, &result, &cache_scope)?;
        db.add_to_history(&validated, &provider_name)?;
        info!(
            "Completed zh-en search for '{}' ({} equivalents)",
            validated,
            result.equivalents.len()
        );
        Ok(result)
    }

    pub async fn search_phrase_progressive(
        &self,
        phrase: &str,
        app: &AppHandle,
        db: &Database,
    ) -> Result<(), AppError> {
        let emitter = PhraseAppHandleEmitter::new(app);
        self.search_phrase_progressive_with_emitter(phrase, db, &emitter)
            .await
    }

    pub async fn search_phrase_progressive_with_emitter<E: PhraseProgressiveEmitter + ?Sized>(
        &self,
        phrase: &str,
        db: &Database,
        emitter: &E,
    ) -> Result<(), AppError> {
        let normalized_phrase = PhraseDetector::normalize_phrase(phrase);
        info!("Progressive 3-section search for phrase: {}", normalized_phrase);

        let settings = self.load_settings(db)?;

        let language = Self::resolve_language(&settings);
        info!("Using explanation language: {}", language);

        let provider_priorities = Self::build_provider_priority_list(&settings);
        if Self::try_emit_cached_phrase(db, &normalized_phrase, emitter, &provider_priorities)? {
            return Ok(());
        }

        let (phrase_service, provider_name) = self.create_phrase_service(&settings)?;

        info!(
            "Launching progressive phrase generation for '{}' (emitting section 1 immediately)",
            normalized_phrase
        );

        let phrase_service = Arc::new(phrase_service);
        let combined = PhraseSectionGenerator::generate_progressive(
            phrase_service,
            &normalized_phrase,
            &language,
            emitter,
        )
        .await?;

        if let Err(e) = db.add_to_history(&normalized_phrase, &provider_name) {
            log::warn!("Failed to add phrase '{}' to history: {}", normalized_phrase, e);
        }

        if let Err(e) = db.cache_phrase(&normalized_phrase, &combined, &provider_name) {
            log::warn!("Failed to cache phrase '{}': {}", normalized_phrase, e);
        }

        info!("Completed 3-section progressive search for phrase: {}", normalized_phrase);

        Ok(())
    }

    pub async fn generate_contextual_examples(
        &self,
        word: &str,
        context: &str,
        db: &Database,
    ) -> Result<Vec<String>, AppError> {
        let validated_word = validation::validate_word_query(word)?;
        let validated_context = validation::validate_context(context)?;
        info!(
            "Generating contextual examples for '{}' in context: {}",
            validated_word, validated_context
        );

        let settings = self.load_settings(db)?;
        let (exploration_service, provider_name) = self.create_exploration_service(&settings)?;

        let (coordinator, _) =
            self.build_explore_coordinator_for_settings(db, &exploration_service, provider_name, &settings);

        coordinator
            .generate_custom_examples(&validated_word, &validated_context)
            .await
    }

    pub async fn generate_formality_analysis(
        &self,
        word: &str,
        db: &Database,
    ) -> Result<(f64, Vec<FormalityAlternative>), AppError> {
        let validated_word = validation::validate_word_query(word)?;
        info!("Generating formality analysis for: {}", validated_word);

        let settings = self.load_settings(db)?;
        let (exploration_service, provider_name) = self.create_exploration_service(&settings)?;

        let language = Self::resolve_language(&settings);
        let cache_provider = provider_cache_key(&provider_name, &language);
        let cache_repo = ExploreCacheRepository::new(db);

        if let Some(cached) = cache_repo.fetch_feature::<CachedFormalityData>(
            &validated_word,
            &cache_provider,
            explore_features::FORMALITY,
        )? {
            info!(
                "Using cached formality analysis for '{}', skipping generation",
                validated_word
            );
            return Ok((cached.formality_percentage, cached.formality_alternatives));
        }

        let (result, _) = exploration_service
            .generate_formality_only(&validated_word, &language)
            .await
            .map_err(|e| {
                log::error!("Formality analysis error for '{}': {}", validated_word, e);
                AppError::from(e)
            })?;

        let formality_percentage = result.formality_percentage;
        let formality_alternatives = result.formality_alternatives;
        let cache_payload = CachedFormalityData {
            formality_percentage,
            formality_alternatives: formality_alternatives.clone(),
        };

        cache_repo.store_feature(
            &validated_word,
            &cache_provider,
            explore_features::FORMALITY,
            &cache_payload,
        );

        Ok((formality_percentage, formality_alternatives))
    }

    pub async fn generate_domain_exploration(
        &self,
        word: &str,
        db: &Database,
    ) -> Result<Vec<DomainExploration>, AppError> {
        let validated_word = validation::validate_word_query(word)?;
        info!(
            "Generating domain exploration for '{}'",
            validated_word
        );

        let settings = self.load_settings(db)?;
        let (exploration_service, provider_name) = self.create_exploration_service(&settings)?;
        let (coordinator, _) =
            self.build_explore_coordinator_for_settings(db, &exploration_service, provider_name, &settings);

        coordinator
            .generate_domain_exploration(&validated_word)
            .await
    }

    pub async fn generate_usage_patterns(
        &self,
        word: &str,
        db: &Database,
    ) -> Result<Vec<UsagePattern>, AppError> {
        let validated_word = validation::validate_word_query(word)?;
        info!("Generating usage patterns for: {}", validated_word);

        let settings = self.load_settings(db)?;
        let (exploration_service, provider_name) = self.create_exploration_service(&settings)?;
        let (coordinator, _) =
            self.build_explore_coordinator_for_settings(db, &exploration_service, provider_name, &settings);

        coordinator.generate_usage_patterns(&validated_word).await
    }

    pub async fn generate_practice_exercises_only(
        &self,
        word: &str,
        count: usize,
        force: bool,
        db: &Database,
    ) -> Result<Vec<PracticeExercise>, AppError> {
        let validated_word = validation::validate_word_query(word)?;
        let validated_count = validation::validate_exercise_count(count)?;
        info!(
            "Generating {} practice exercises for: {}",
            validated_count, validated_word
        );

        let settings = self.load_settings(db)?;
        let (exploration_service, provider_name) = self.create_exploration_service(&settings)?;
        let (coordinator, _) =
            self.build_explore_coordinator_for_settings(db, &exploration_service, provider_name, &settings);

        coordinator
            .generate_practice_exercises(&validated_word, validated_count, force)
            .await
    }

    pub async fn generate_common_mistakes(
        &self,
        word: &str,
        force: bool,
        db: &Database,
    ) -> Result<Vec<MistakeItem>, AppError> {
        let validated_word = validation::validate_word_query(word)?;
        info!("Generating common mistakes for: {}", validated_word);

        let settings = self.load_settings(db)?;
        let (dictionary_service, provider_name) = self.create_dictionary_service(&settings)?;
        let (exploration_service, _) = self.create_exploration_service(&settings)?;

        let language = Self::resolve_language(&settings);
        let provider_scope = provider_cache_key(&provider_name, &language);
        let cache_repo = ExploreCacheRepository::new(db);
        let coordinator = ExploreFeatureCoordinator::new(
            cache_repo,
            provider_scope,
            provider_name,
            language,
            &exploration_service,
            db,
        );

        coordinator
            .generate_mistakes(&validated_word, &dictionary_service, force)
            .await
    }

    pub async fn fetch_available_models(
        &self,
        provider: &str,
        credential_or_endpoint: &str,
        db: &Database,
    ) -> Result<Vec<AiModel>, AppError> {
        if provider == providers::OLLAMA {
            let endpoint = self.resolve_ollama_endpoint(credential_or_endpoint, db)?;
            return self
                .provider_resolver
                .ollama_service()
                .fetch_ai_models(&endpoint)
                .await;
        }

        self.provider_resolver
            .fetch_models(provider, credential_or_endpoint)
            .await
    }

    pub async fn test_api_key(
        &self,
        provider: &str,
        credential_or_endpoint: &str,
        db: &Database,
    ) -> Result<bool, AppError> {
        if provider == providers::OLLAMA {
            let endpoint = self.resolve_ollama_endpoint(credential_or_endpoint, db)?;
            return Ok(self
                .provider_resolver
                .ollama_service()
                .detect(&endpoint)
                .await);
        }

        self.provider_resolver
            .test_api_key(provider, credential_or_endpoint)
            .await
    }

    pub async fn detect_ollama(&self, db: &Database) -> Result<bool, AppError> {
        let endpoint = self.default_ollama_endpoint(db)?;
        Ok(self
            .provider_resolver
            .ollama_service()
            .detect(&endpoint)
            .await)
    }

    pub async fn list_ollama_models(&self, db: &Database) -> Result<Vec<String>, AppError> {
        let endpoint = self.default_ollama_endpoint(db)?;
        self.provider_resolver
            .ollama_service()
            .list_models(&endpoint)
            .await
    }

    fn default_ollama_endpoint(&self, db: &Database) -> Result<String, AppError> {
        let settings = self.load_settings(db)?;
        Ok(self.settings_service.extract_ollama_endpoint(&settings))
    }

    fn resolve_ollama_endpoint(&self, input: &str, db: &Database) -> Result<String, AppError> {
        if input.trim().is_empty() {
            self.default_ollama_endpoint(db)
        } else {
            Ok(crate::services::ollama::normalize_ollama_endpoint(Some(
                input,
            )))
        }
    }

    pub fn update_ai_provider(
        &self,
        provider: &str,
        config: serde_json::Value,
        db: &Database,
    ) -> Result<(), AppError> {
        let mut settings = self.load_settings(db)?;
        self.settings_service
            .apply_provider_config(provider, config, &mut settings)?;
        db.save_settings(&settings)?;
        self.provider_resolver.invalidate_cache();
        Ok(())
    }

    pub fn get_cached_exploration_features(
        &self,
        word: &str,
        db: &Database,
    ) -> Result<CachedExploreFeatures, AppError> {
        let validated_word = validation::validate_word_query(word)?;
        info!(
            "Fetching cached exploration features for '{}'",
            validated_word
        );
        let settings = self.load_settings(db)?;
        let provider_name = settings.ai_provider.clone();
        let language = Self::resolve_language(&settings);
        let cache_provider = provider_cache_key(&provider_name, &language);
        log::debug!(
            "Using provider '{}' (language='{}') for explore cache lookups on '{}'",
            provider_name,
            language,
            validated_word
        );
        let cache_repo = ExploreCacheRepository::new(db);
        let cached = cache_repo.load_all(&validated_word, &cache_provider)?;

        let domains_count = cached.domains.as_ref().map(|d| d.len()).unwrap_or(0);
        let usage_count = cached.usage.as_ref().map(|u| u.len()).unwrap_or(0);
        let practice_exercise_count = cached
            .practice
            .as_ref()
            .map(|p| p.practice_exercises.len())
            .unwrap_or(0);
        info!(
            "Explore cache lookup completed for '{}': formality_hit={}, domains={}, usage={}, practice_exercises={}",
            validated_word,
            cached.formality.is_some(),
            domains_count,
            usage_count,
            practice_exercise_count
        );

        Ok(cached)
    }
}

fn provider_cache_key(provider: &str, language: &str) -> String {
    let normalized_language = language.trim().to_lowercase().replace(' ', "-");
    format!("{}::{}", provider, normalized_language)
}

impl Default for CommandOrchestrator {
    fn default() -> Self {
        Self::new(ProviderRegistry::default())
    }
}
