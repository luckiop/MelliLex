use crate::errors::AppError;
use crate::models::VocabEntry;
use crate::utils::sync;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

const LIST_LIMIT: i32 = 2000;

#[derive(Clone)]
pub struct VocabRepository {
    conn: Arc<Mutex<Connection>>,
}

impl VocabRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn get(&self, word_key: &str) -> Result<Option<VocabEntry>, AppError> {
        let conn = sync::lock(&self.conn, "vocab connection")?;
        let mut stmt = conn.prepare(
            "SELECT word, stars, category, added_at, updated_at
             FROM vocab_entries
             WHERE word_key = ?1",
        )?;

        let entry = stmt
            .query_row(params![word_key], map_vocab_row)
            .optional()?;
        Ok(entry)
    }

    pub fn list(
        &self,
        stars: Option<i32>,
        category: Option<&str>,
    ) -> Result<Vec<VocabEntry>, AppError> {
        let conn = sync::lock(&self.conn, "vocab connection")?;

        let mut entries = Vec::new();
        match (stars, category) {
            (None, None) => {
                let mut stmt = conn.prepare(
                    "SELECT word, stars, category, added_at, updated_at
                     FROM vocab_entries
                     ORDER BY updated_at DESC
                     LIMIT ?1",
                )?;
                let rows = stmt.query_map(params![LIST_LIMIT], map_vocab_row)?;
                for row in rows {
                    entries.push(row?);
                }
            }
            (Some(stars), None) => {
                let mut stmt = conn.prepare(
                    "SELECT word, stars, category, added_at, updated_at
                     FROM vocab_entries
                     WHERE stars = ?1
                     ORDER BY updated_at DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![stars, LIST_LIMIT], map_vocab_row)?;
                for row in rows {
                    entries.push(row?);
                }
            }
            (None, Some(category)) => {
                let mut stmt = conn.prepare(
                    "SELECT word, stars, category, added_at, updated_at
                     FROM vocab_entries
                     WHERE category = ?1
                     ORDER BY updated_at DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![category, LIST_LIMIT], map_vocab_row)?;
                for row in rows {
                    entries.push(row?);
                }
            }
            (Some(stars), Some(category)) => {
                let mut stmt = conn.prepare(
                    "SELECT word, stars, category, added_at, updated_at
                     FROM vocab_entries
                     WHERE stars = ?1 AND category = ?2
                     ORDER BY updated_at DESC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(params![stars, category, LIST_LIMIT], map_vocab_row)?;
                for row in rows {
                    entries.push(row?);
                }
            }
        }

        Ok(entries)
    }

    /// Insert or update. Returns the stored entry and whether this was a first insert.
    pub fn upsert(
        &self,
        word_key: &str,
        word: &str,
        stars: i32,
        category: &str,
    ) -> Result<(VocabEntry, bool), AppError> {
        let existed = self.get(word_key)?.is_some();
        let conn = sync::lock(&self.conn, "vocab connection")?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO vocab_entries (word_key, word, stars, category, added_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(word_key) DO UPDATE SET
                word = excluded.word,
                stars = excluded.stars,
                category = excluded.category,
                updated_at = excluded.updated_at",
            params![word_key, word, stars, category, now],
        )?;

        let added_at: i64 = conn.query_row(
            "SELECT added_at FROM vocab_entries WHERE word_key = ?1",
            params![word_key],
            |row| row.get(0),
        )?;

        Ok((
            VocabEntry {
                word: word.to_string(),
                stars,
                category: category.to_string(),
                added_at,
                updated_at: now,
            },
            !existed,
        ))
    }

    pub fn delete(&self, word_key: &str) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "vocab connection")?;
        let rows = conn.execute(
            "DELETE FROM vocab_entries WHERE word_key = ?1",
            params![word_key],
        )?;
        if rows == 0 {
            return Err(AppError::not_found("Vocab entry not found"));
        }
        Ok(())
    }
}

fn map_vocab_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabEntry> {
    Ok(VocabEntry {
        word: row.get(0)?,
        stars: row.get(1)?,
        category: row.get(2)?,
        added_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}
