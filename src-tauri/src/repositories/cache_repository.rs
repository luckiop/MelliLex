use crate::errors::AppError;
use crate::models::{AiWordDefinition, WordProgressiveData};
use crate::utils::sync;
use rusqlite::{params, Connection};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CacheRepository {
    conn: Arc<Mutex<Connection>>,
}

impl CacheRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn definition(
        &self,
        word: &str,
        provider: &str,
    ) -> Result<Option<AiWordDefinition>, AppError> {
        let conn = sync::lock(&self.conn, "cached definitions connection")?;
        let cache_key = format!("{}:{}", provider, word.to_lowercase());

        let mut stmt = conn
            .prepare("SELECT definition FROM cached_definitions WHERE cache_key = ?1")
            .map_err(AppError::from)?;

        let result = stmt.query_row([cache_key], |row| {
            let definition_str: String = row.get(0)?;
            Ok(definition_str)
        });

        match result {
            Ok(definition_str) => {
                let definition: AiWordDefinition =
                    serde_json::from_str(&definition_str).map_err(|e| {
                        AppError::Parse(format!("Failed to deserialize cached definition: {}", e))
                    })?;
                Ok(Some(definition))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn cache_definition(
        &self,
        word: &str,
        definition: &AiWordDefinition,
        provider: &str,
    ) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "cached definitions connection")?;
        let cache_key = format!("{}:{}", provider, word.to_lowercase());
        let definition_json = serde_json::to_string(definition)?;
        let cached_at = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO cached_definitions (cache_key, word, definition, provider, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![cache_key, word, definition_json, provider, cached_at],
        )
        .map_err(AppError::from)?;

        Ok(())
    }

    pub fn get_cache_stats(&self) -> Result<(usize, usize, i64), AppError> {
        let conn = sync::lock(&self.conn, "cache stats connection")?;

        let definition_count: usize = conn
            .query_row("SELECT COUNT(*) FROM cached_definitions", [], |row| {
                row.get(0)
            })
            .map_err(AppError::from)?;

        let exploration_count: usize = conn
            .query_row("SELECT COUNT(*) FROM cached_explorations", [], |row| {
                row.get(0)
            })
            .map_err(AppError::from)?;

        // Get total size estimate (rough approximation based on JSON length)
        let total_size: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(LENGTH(definition)), 0) FROM cached_definitions",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            + conn
                .query_row(
                    "SELECT COALESCE(SUM(LENGTH(exploration_data)), 0) FROM cached_explorations",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

        Ok((definition_count, exploration_count, total_size))
    }

    pub fn clear_all_cache(&self) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "clear cache connection")?;

        conn.execute("DELETE FROM cached_definitions", [])
            .map_err(AppError::from)?;

        conn.execute("DELETE FROM cached_explorations", [])
            .map_err(AppError::from)?;

        Ok(())
    }

    pub fn clear_definition_cache(&self) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "clear definition cache connection")?;
        conn.execute("DELETE FROM cached_definitions", [])
            .map_err(AppError::from)?;
        Ok(())
    }

    pub fn clear_exploration_cache(&self) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "clear exploration cache connection")?;
        conn.execute("DELETE FROM cached_explorations", [])
            .map_err(AppError::from)?;
        Ok(())
    }

    pub fn clear_old_cache(&self, days: i64) -> Result<usize, AppError> {
        let conn = sync::lock(&self.conn, "clear old cache connection")?;
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 86400);

        let def_deleted = conn
            .execute(
                "DELETE FROM cached_definitions WHERE cached_at < ?1",
                params![cutoff_time],
            )
            .map_err(AppError::from)?;

        let exp_deleted = conn
            .execute(
                "DELETE FROM cached_explorations WHERE cached_at < ?1",
                params![cutoff_time],
            )
            .map_err(AppError::from)?;

        Ok(def_deleted + exp_deleted)
    }

    pub fn word_progressive(
        &self,
        word: &str,
        provider: &str,
    ) -> Result<Option<WordProgressiveData>, AppError> {
        let cache_key = format!("{}:wordprogressive:{}", provider, word.to_lowercase());
        let conn = sync::lock(&self.conn, "word progressive cache connection")?;

        let result = conn.query_row(
            "SELECT definition FROM cached_definitions WHERE cache_key = ? LIMIT 1",
            [&cache_key],
            |row| {
                let json: String = row.get(0)?;
                Ok(json)
            },
        );

        match result {
            Ok(json) => {
                let data: WordProgressiveData = serde_json::from_str(&json).map_err(|e| {
                    AppError::Parse(format!("Failed to parse cached word progressive data: {}", e))
                })?;
                Ok(Some(data))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn cache_word_progressive(
        &self,
        word: &str,
        data: &WordProgressiveData,
        provider: &str,
    ) -> Result<(), AppError> {
        let cache_key = format!("{}:wordprogressive:{}", provider, word.to_lowercase());
        let json = serde_json::to_string(data)
            .map_err(|e| AppError::Parse(format!("Failed to serialize word progressive data: {}", e)))?;
        let cached_at = chrono::Utc::now().timestamp();

        let conn = sync::lock(&self.conn, "cache word progressive connection")?;
        conn.execute(
            "INSERT OR REPLACE INTO cached_definitions (cache_key, word, definition, provider, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&cache_key, word, &json, provider, cached_at],
        )
        .map_err(AppError::from)?;

        Ok(())
    }

    fn zh_en_cache_key(provider: &str, query: &str) -> String {
        format!("{}:zhen:{}", provider, query.trim())
    }

    pub fn zh_en(
        &self,
        query: &str,
        provider: &str,
    ) -> Result<Option<crate::models::ZhEnResult>, AppError> {
        let cache_key = Self::zh_en_cache_key(provider, query);
        let conn = sync::lock(&self.conn, "zh-en cache connection")?;
        let result = conn.query_row(
            "SELECT definition FROM cached_definitions WHERE cache_key = ? LIMIT 1",
            [&cache_key],
            |row| {
                let json: String = row.get(0)?;
                Ok(json)
            },
        );

        match result {
            Ok(json) => {
                let data = serde_json::from_str(&json).map_err(|e| {
                    AppError::Parse(format!("Failed to parse cached zh-en data: {}", e))
                })?;
                Ok(Some(data))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn cache_zh_en(
        &self,
        query: &str,
        data: &crate::models::ZhEnResult,
        provider: &str,
    ) -> Result<(), AppError> {
        let cache_key = Self::zh_en_cache_key(provider, query);
        let json = serde_json::to_string(data)
            .map_err(|e| AppError::Parse(format!("Failed to serialize zh-en data: {}", e)))?;
        let cached_at = chrono::Utc::now().timestamp();
        let conn = sync::lock(&self.conn, "cache zh-en connection")?;
        conn.execute(
            "INSERT OR REPLACE INTO cached_definitions (cache_key, word, definition, provider, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&cache_key, query, &json, provider, cached_at],
        )
        .map_err(AppError::from)?;
        Ok(())
    }

    fn translate_cache_key(provider: &str, query: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query.hash(&mut hasher);
        format!("{}:translate:{:x}", provider, hasher.finish())
    }

    pub fn translate(
        &self,
        query: &str,
        provider: &str,
    ) -> Result<Option<crate::models::TranslateResult>, AppError> {
        let cache_key = Self::translate_cache_key(provider, query);
        let conn = sync::lock(&self.conn, "translate cache connection")?;
        let result = conn.query_row(
            "SELECT definition FROM cached_definitions WHERE cache_key = ? LIMIT 1",
            [&cache_key],
            |row| {
                let json: String = row.get(0)?;
                Ok(json)
            },
        );

        match result {
            Ok(json) => {
                let data = serde_json::from_str(&json).map_err(|e| {
                    AppError::Parse(format!("Failed to parse cached translate data: {}", e))
                })?;
                Ok(Some(data))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn cache_translate(
        &self,
        query: &str,
        data: &crate::models::TranslateResult,
        provider: &str,
    ) -> Result<(), AppError> {
        let cache_key = Self::translate_cache_key(provider, query);
        let json = serde_json::to_string(data)
            .map_err(|e| AppError::Parse(format!("Failed to serialize translate data: {}", e)))?;
        let cached_at = chrono::Utc::now().timestamp();
        let label: String = query.chars().take(80).collect();
        let conn = sync::lock(&self.conn, "cache translate connection")?;
        conn.execute(
            "INSERT OR REPLACE INTO cached_definitions (cache_key, word, definition, provider, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&cache_key, label, &json, provider, cached_at],
        )
        .map_err(AppError::from)?;
        Ok(())
    }

    fn exploration_cache_key(provider: &str, feature: &str, word: &str) -> String {
        format!("{}:explore:{}:{}", provider, feature, word.to_lowercase())
    }

    pub fn get_cached_exploration_feature<T: DeserializeOwned>(
        &self,
        word: &str,
        provider: &str,
        feature: &str,
    ) -> Result<Option<T>, AppError> {
        let cache_key = Self::exploration_cache_key(provider, feature, word);
        let conn = sync::lock(&self.conn, "get cached exploration connection")?;

        let result = conn.query_row(
            "SELECT exploration_data FROM cached_explorations WHERE cache_key = ?1",
            [&cache_key],
            |row| {
                let json: String = row.get(0)?;
                Ok(json)
            },
        );

        match result {
            Ok(json) => {
                let data: T = serde_json::from_str(&json).map_err(|e| {
                    AppError::Parse(format!(
                        "Failed to deserialize cached exploration data: {}",
                        e
                    ))
                })?;
                Ok(Some(data))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn cache_exploration_feature<T: Serialize>(
        &self,
        word: &str,
        provider: &str,
        feature: &str,
        data: &T,
    ) -> Result<(), AppError> {
        let cache_key = Self::exploration_cache_key(provider, feature, word);
        let json = serde_json::to_string(data).map_err(|e| {
            AppError::Parse(format!(
                "Failed to serialize exploration cache payload ({}:{}): {}",
                feature, word, e
            ))
        })?;
        let cached_at = chrono::Utc::now().timestamp();

        let conn = sync::lock(&self.conn, "cache exploration feature connection")?;
        conn.execute(
            "INSERT OR REPLACE INTO cached_explorations (cache_key, word, exploration_data, provider, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![cache_key, word, json, provider, cached_at],
        )
        .map_err(AppError::from)?;

        Ok(())
    }
}
