use crate::errors::AppError;
use rusqlite::Connection;

/// Single Responsibility: Create and manage database schema (DDL operations)
pub struct SchemaManager;

impl SchemaManager {
    pub fn init_schema(conn: &Connection) -> Result<(), AppError> {
        log::debug!("Creating database schema");

        conn.execute("PRAGMA foreign_keys = ON", [])?;

        Self::create_word_history_table(conn)?;
        Self::create_cache_tables(conn)?;
        Self::create_settings_tables(conn)?;
        Self::create_phrase_tables(conn)?;
        Self::create_vocab_table(conn)?;

        Ok(())
    }

    fn create_word_history_table(conn: &Connection) -> Result<(), AppError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS word_history (
                id TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                context TEXT DEFAULT 'main_search',
                ai_provider TEXT DEFAULT 'cloud',
                exploration_path TEXT,
                phonetic TEXT,
                short_definition TEXT,
                complexity TEXT DEFAULT 'intermediate',
                is_idiom INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_timestamp 
             ON word_history(timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    fn create_cache_tables(conn: &Connection) -> Result<(), AppError> {
        // Cached Definitions Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_definitions (
                cache_key TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                definition TEXT NOT NULL,
                provider TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Index for efficient cache lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_definitions_cache_key 
             ON cached_definitions(cache_key)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_definitions_cached_at 
             ON cached_definitions(cached_at DESC)",
            [],
        )?;

        // Cached Explorations Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_explorations (
                cache_key TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                exploration_data TEXT NOT NULL,
                provider TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Index for efficient cache lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_explorations_cache_key 
             ON cached_explorations(cache_key)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_explorations_cached_at 
             ON cached_explorations(cached_at DESC)",
            [],
        )?;

        Ok(())
    }

    fn create_settings_tables(conn: &Connection) -> Result<(), AppError> {
        // Settings Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Secure Settings Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS secure_settings (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                nonce BLOB NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    fn create_phrase_tables(conn: &Connection) -> Result<(), AppError> {
        // Phrase History Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS phrase_history (
                id TEXT PRIMARY KEY,
                phrase TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                context TEXT DEFAULT 'main_search',
                ai_provider TEXT DEFAULT 'cloud'
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_phrase_history_timestamp 
             ON phrase_history(timestamp DESC)",
            [],
        )?;

        // Cached Phrase Definitions Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_phrases (
                cache_key TEXT PRIMARY KEY,
                phrase TEXT NOT NULL,
                definition TEXT NOT NULL,
                provider TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Index for efficient cache lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_phrases_cache_key 
             ON cached_phrases(cache_key)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_phrases_cached_at 
             ON cached_phrases(cached_at DESC)",
            [],
        )?;

        // Saved Phrases Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS saved_phrases (
                id TEXT PRIMARY KEY,
                phrase TEXT NOT NULL UNIQUE,
                phrase_type TEXT NOT NULL,
                tldr TEXT NOT NULL,
                notes TEXT,
                saved_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_saved_phrases_date 
             ON saved_phrases(saved_at DESC)",
            [],
        )?;

        Ok(())
    }

    fn create_vocab_table(conn: &Connection) -> Result<(), AppError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vocab_entries (
                word_key TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                stars INTEGER NOT NULL,
                category TEXT NOT NULL,
                added_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vocab_updated
             ON vocab_entries(updated_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vocab_filter
             ON vocab_entries(category, stars)",
            [],
        )?;

        Ok(())
    }
}
