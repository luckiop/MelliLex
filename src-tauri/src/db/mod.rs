mod schema_manager;

pub use schema_manager::SchemaManager;

use crate::errors::AppError;
use crate::models::{
    AiWordDefinition, AppSettings, PhraseDefinitionData, VocabEntry, WordProgressiveData,
    WordHistory,
};
use crate::repositories::cache_repository::CacheRepository;
use crate::repositories::history_repository::HistoryRepository;
use crate::repositories::phrase_cache_repository::PhraseCacheRepository;
use crate::repositories::settings_repository::SettingsRepository;
use crate::repositories::vocab_repository::VocabRepository;
use crate::security::secret_store::SecretStore;
use crate::utils::sync;
use rusqlite::Connection;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Single Responsibility: Coordinate database operations via specialized repositories
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    history_repo: HistoryRepository,
    cache_repo: CacheRepository,
    phrase_cache_repo: PhraseCacheRepository,
    settings_repo: SettingsRepository,
    vocab_repo: VocabRepository,
    secret_store: SecretStore,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self, AppError> {
        log::info!("Initializing database at: {:?}", db_path);

        let conn = Connection::open(&db_path).map_err(|e| {
            log::error!("Failed to open database: {}", e);
            AppError::from(e)
        })?;

        let key_path = db_path.with_extension("key");

        let conn = Arc::new(Mutex::new(conn));
        let secret_store = SecretStore::new(conn.clone(), &key_path)?;

        let db = Database {
            conn: conn.clone(),
            history_repo: HistoryRepository::new(conn.clone()),
            cache_repo: CacheRepository::new(conn.clone()),
            phrase_cache_repo: PhraseCacheRepository::new(conn.clone()),
            settings_repo: SettingsRepository::new(conn.clone()),
            vocab_repo: VocabRepository::new(conn.clone()),
            secret_store,
        };

        // Initialize schema, migrations, and seed data
        {
            let conn_guard = sync::lock(&db.conn, "database initialization connection")?;
            SchemaManager::init_schema(&conn_guard)?;
        }

        log::info!("Database initialized successfully");

        Ok(db)
    }

    // ============== HISTORY OPERATIONS ==============

    pub fn add_to_history(&self, word: &str, ai_provider: &str) -> Result<WordHistory, AppError> {
        self.history_repo.add(word, ai_provider)
    }

    pub fn get_history(&self, limit: i32) -> Result<Vec<WordHistory>, AppError> {
        self.history_repo.list(limit)
    }

    pub fn delete_history_item(&self, id: &str) -> Result<(), AppError> {
        self.history_repo.delete(id)
    }

    pub fn clear_history(&self) -> Result<(), AppError> {
        self.history_repo.clear()
    }

    // ============== CACHE OPERATIONS ==============

    pub fn get_cached_definition(
        &self,
        word: &str,
        provider: &str,
    ) -> Result<Option<AiWordDefinition>, AppError> {
        self.cache_repo.definition(word, provider)
    }

    pub fn cache_definition(
        &self,
        word: &str,
        definition: &AiWordDefinition,
        provider: &str,
    ) -> Result<(), AppError> {
        self.cache_repo.cache_definition(word, definition, provider)
    }

    pub fn get_cached_word_progressive(
        &self,
        word: &str,
        provider: &str,
    ) -> Result<Option<WordProgressiveData>, AppError> {
        self.cache_repo.word_progressive(word, provider)
    }

    pub fn cache_word_progressive(
        &self,
        word: &str,
        data: &WordProgressiveData,
        provider: &str,
    ) -> Result<(), AppError> {
        self.cache_repo
            .cache_word_progressive(word, data, provider)
    }

    pub fn get_cached_exploration_feature<T: DeserializeOwned>(
        &self,
        word: &str,
        provider: &str,
        feature: &str,
    ) -> Result<Option<T>, AppError> {
        self.cache_repo
            .get_cached_exploration_feature(word, provider, feature)
    }

    pub fn cache_exploration_feature<T: Serialize>(
        &self,
        word: &str,
        provider: &str,
        feature: &str,
        data: &T,
    ) -> Result<(), AppError> {
        self.cache_repo
            .cache_exploration_feature(word, provider, feature, data)
    }

    // ============== PHRASE CACHE OPERATIONS ==============

    pub fn get_cached_phrase(
        &self,
        phrase: &str,
        provider: &str,
    ) -> Result<Option<PhraseDefinitionData>, AppError> {
        self.phrase_cache_repo.get(phrase, provider)
    }

    pub fn cache_phrase(
        &self,
        phrase: &str,
        data: &PhraseDefinitionData,
        provider: &str,
    ) -> Result<(), AppError> {
        self.phrase_cache_repo.cache(phrase, data, provider)
    }

    // ============== SETTINGS OPERATIONS ==============

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let mut settings = self.settings_repo.load()?;
        self.secret_store.hydrate_provider_secrets(&mut settings)?;
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.secret_store.persist_provider_secrets(settings)?;
        let sanitized = SecretStore::sanitize_settings(settings);
        self.settings_repo.save(&sanitized)
    }

    // ============== VOCAB OPERATIONS ==============

    pub fn get_vocab_entry(&self, word: &str) -> Result<Option<VocabEntry>, AppError> {
        self.vocab_repo.get(&word.to_lowercase())
    }

    pub fn list_vocab_entries(
        &self,
        stars: Option<i32>,
        category: Option<&str>,
    ) -> Result<Vec<VocabEntry>, AppError> {
        self.vocab_repo.list(stars, category)
    }

    pub fn upsert_vocab_entry(
        &self,
        word: &str,
        stars: i32,
        category: &str,
    ) -> Result<(VocabEntry, bool), AppError> {
        self.vocab_repo
            .upsert(&word.to_lowercase(), word, stars, category)
    }

    pub fn delete_vocab_entry(&self, word: &str) -> Result<(), AppError> {
        self.vocab_repo.delete(&word.to_lowercase())
    }

    // ============== CACHE STATS & CLEANUP ==============

    pub fn get_cache_stats(&self) -> Result<(usize, usize, usize, i64), AppError> {
        let (definitions, explorations, word_size) = self.cache_repo.get_cache_stats()?;
        let (phrases, phrase_size) = self.phrase_cache_repo.get_stats()?;
        let total_size = word_size + phrase_size;
        Ok((definitions, explorations, phrases, total_size))
    }

    pub fn clear_all_cache(&self) -> Result<(), AppError> {
        self.cache_repo.clear_all_cache()?;
        self.phrase_cache_repo.clear_all()?;
        Ok(())
    }

    pub fn clear_definition_cache(&self) -> Result<(), AppError> {
        self.cache_repo.clear_definition_cache()?;
        self.phrase_cache_repo.clear_all()?; // Phrases are definitions too
        Ok(())
    }

    pub fn clear_exploration_cache(&self) -> Result<(), AppError> {
        self.cache_repo.clear_exploration_cache()
    }

    pub fn clear_old_cache(&self, days: i64) -> Result<usize, AppError> {
        let word_deleted = self.cache_repo.clear_old_cache(days)?;
        let phrase_deleted = self.phrase_cache_repo.clear_old(days)?;
        Ok(word_deleted + phrase_deleted)
    }
}

#[cfg(test)]
mod tests {
    include!("db_tests.rs");
}
