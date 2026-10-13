//! 0.13.0 storage view. Map: `src/release/v0_13_0.ts`.
//!
//! Lookup text, voice wavs, and logs can each be deleted by age. The voice
//! model, speech libraries, and database file are measured only. Lookup rows
//! sit inside the database, so a lookup delete is followed by VACUUM.

use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::db::{Database, LookupUsage};
use crate::errors::AppError;
use crate::services::kokoro;
use crate::services::voice_cache;

const AGE_DAYS: [u64; 3] = [7, 30, 90];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageReport {
    pub total_bytes: u64,
    pub lookups: CacheSlice,
    pub voice: CacheSlice,
    pub logs: CacheSlice,
    pub model_bytes: u64,
    pub runtime_bytes: u64,
    pub database_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheSlice {
    pub bytes: u64,
    pub count: u64,
    pub oldest_unix: Option<i64>,
    pub older7: u64,
    pub older30: u64,
    pub older90: u64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageClearResult {
    pub removed: u64,
    pub skipped: u64,
}

pub fn report(db: &Database, app_data: &Path, log_dir: &Path) -> Result<StorageReport, AppError> {
    let now = chrono::Utc::now().timestamp();
    let cutoffs = AGE_DAYS.map(|days| now - (days as i64 * 86_400));
    let lookups = cache_slice(db.lookup_usage(cutoffs)?);
    let voice = files_slice(&voice_cache::list_wavs(&voice_cache::dir(app_data)));
    let logs = files_slice(&list_files(log_dir));
    let model_bytes = dir_bytes(&kokoro::model_dir(app_data));
    let runtime_bytes = dir_bytes(&kokoro::runtime_dir(app_data));
    let database_bytes = database_bytes(app_data);
    let total_bytes = database_bytes + voice.bytes + logs.bytes + model_bytes + runtime_bytes;
    Ok(StorageReport {
        total_bytes,
        lookups,
        voice,
        logs,
        model_bytes,
        runtime_bytes,
        database_bytes,
    })
}

pub fn clear(
    db: &Database,
    app_data: &Path,
    log_dir: &Path,
    kind: &str,
    older_than_days: Option<u32>,
) -> Result<StorageClearResult, AppError> {
    let days = match older_than_days {
        Some(days) if AGE_DAYS.contains(&(u64::from(days))) => Some(u64::from(days)),
        Some(_) => return Err(AppError::validation("Choose 7, 30, or 90 days")),
        None => None,
    };
    match kind {
        "lookups" => clear_lookups(db, days),
        "voice" => Ok(clear_files(&voice_cache::list_wavs(&voice_cache::dir(app_data)), days, false)),
        "logs" => Ok(clear_files(&list_files(log_dir), days, true)),
        _ => Err(AppError::validation("Unknown storage category")),
    }
}

fn clear_lookups(db: &Database, days: Option<u64>) -> Result<StorageClearResult, AppError> {
    let removed = if let Some(days) = days {
        db.clear_old_cache(days as i64)? as u64
    } else {
        let usage = db.lookup_usage([0, 0, 0])?;
        db.clear_all_cache()?;
        usage.count
    };
    if let Err(err) = db.compact() {
        log::warn!("Database file did not shrink after clearing lookups: {err}");
    }
    Ok(StorageClearResult { removed, skipped: 0 })
}

fn clear_files(files: &[(std::path::PathBuf, SystemTime, u64)], days: Option<u64>, truncate_busy: bool) -> StorageClearResult {
    let cutoff = days.map(|days| SystemTime::now() - Duration::from_secs(days * 86_400));
    let mut removed = 0u64;
    let mut skipped = 0u64;
    for (path, modified, _) in files {
        if cutoff.is_some_and(|cutoff| *modified >= cutoff) {
            continue;
        }
        match fs::remove_file(path) {
            Ok(()) => removed += 1,
            Err(_) if truncate_busy => match File::create(path) {
                Ok(_) => removed += 1,
                Err(_) => skipped += 1,
            },
            Err(_) => skipped += 1,
        }
    }
    StorageClearResult { removed, skipped }
}

fn cache_slice(usage: LookupUsage) -> CacheSlice {
    CacheSlice {
        bytes: usage.bytes,
        count: usage.count,
        oldest_unix: usage.oldest_unix,
        older7: usage.older_bytes[0],
        older30: usage.older_bytes[1],
        older90: usage.older_bytes[2],
    }
}

fn files_slice(files: &[(std::path::PathBuf, SystemTime, u64)]) -> CacheSlice {
    let now = SystemTime::now();
    let mut slice = CacheSlice {
        bytes: 0,
        count: 0,
        oldest_unix: None,
        older7: 0,
        older30: 0,
        older90: 0,
    };
    for (_, modified, len) in files {
        slice.bytes += len;
        slice.count += 1;
        let unix = modified.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs() as i64);
        slice.oldest_unix = match (slice.oldest_unix, unix) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(left), None) => Some(left),
            (None, right) => right,
        };
        let age = now.duration_since(*modified).unwrap_or_default();
        if age >= Duration::from_secs(7 * 86_400) {
            slice.older7 += len;
        }
        if age >= Duration::from_secs(30 * 86_400) {
            slice.older30 += len;
        }
        if age >= Duration::from_secs(90 * 86_400) {
            slice.older90 += len;
        }
    }
    slice
}

fn list_files(dir: &Path) -> Vec<(std::path::PathBuf, SystemTime, u64)> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        files.push((
            entry.path(),
            meta.modified().unwrap_or(UNIX_EPOCH),
            meta.len(),
        ));
    }
    files
}

fn dir_bytes(dir: &Path) -> u64 {
    fn walk(dir: &Path, total: &mut u64) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                walk(&path, total);
            } else if meta.is_file() {
                *total += meta.len();
            }
        }
    }
    let mut total = 0;
    walk(dir, &mut total);
    total
}

fn database_bytes(app_data: &Path) -> u64 {
    ["mellilex.db", "mellilex.db-wal", "mellilex.db-shm"]
        .into_iter()
        .filter_map(|name| fs::metadata(app_data.join(name)).ok())
        .map(|meta| meta.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AiMeaning, AiWordDefinition, Definition};

    fn sample() -> AiWordDefinition {
        AiWordDefinition {
            word: "sample".into(),
            phonetic: None,
            domain_tags: vec![],
            complexity: None,
            frequency: None,
            etymology: None,
            syllable_info: None,
            metrics: None,
            meanings: vec![AiMeaning {
                part_of_speech: "noun".into(),
                definitions: vec![Definition {
                    text: "an example kept on disk".into(),
                    examples: vec![],
                    contextual_examples: vec![],
                    memory_tip: None,
                    confidence: None,
                }],
                synonyms: vec![],
                antonyms: vec![],
                collocations: vec![],
            }],
            common_mistakes: vec![],
            contextual_usage: None,
            token_usage: None,
        }
    }

    #[test]
    fn clearing_old_voice_files_leaves_a_recent_recording() {
        let root = tempfile::tempdir().unwrap();
        let cache = voice_cache::dir(root.path());
        fs::create_dir_all(&cache).unwrap();
        let old = cache.join("old.wav");
        let recent = cache.join("recent.wav");
        fs::write(&old, b"RIFF old").unwrap();
        fs::write(&recent, b"RIFF new").unwrap();
        let stale = SystemTime::now() - Duration::from_secs(40 * 86_400);
        File::options().write(true).open(&old).unwrap().set_modified(stale).unwrap();

        let db_dir = tempfile::tempdir().unwrap();
        let db = Database::new(db_dir.path().join("mellilex.db")).unwrap();
        let result = clear(&db, root.path(), &root.path().join("logs"), "voice", Some(30)).unwrap();

        assert_eq!(result.removed, 1);
        assert!(recent.is_file());
        assert!(!old.exists());
    }

    #[test]
    fn lookup_usage_counts_only_rows_older_than_the_cutoff() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::new(dir.path().join("mellilex.db")).unwrap();
        db.cache_definition("kept", &sample(), "openai").unwrap();
        let usage = db.lookup_usage([0, 0, 0]).unwrap();
        assert_eq!(usage.count, 1);
        assert!(usage.bytes > 0);

        let cleared = clear(&db, dir.path(), &dir.path().join("logs"), "lookups", None).unwrap();
        assert_eq!(cleared.removed, 1);
        assert_eq!(db.lookup_usage([0, 0, 0]).unwrap().count, 0);
    }

    #[test]
    fn an_old_lookup_can_be_removed_without_the_recent_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mellilex.db");
        let db = Database::new(path.clone()).unwrap();
        db.cache_definition("old", &sample(), "openai").unwrap();
        db.cache_definition("new", &sample(), "openai").unwrap();
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute(
                "UPDATE cached_definitions SET cached_at = ?1 WHERE word = 'old'",
                [chrono::Utc::now().timestamp() - 40 * 86_400],
            )
            .unwrap();
        }

        let removed = clear(&db, dir.path(), &dir.path().join("missing-logs"), "lookups", Some(30)).unwrap();
        assert_eq!(removed.removed, 1);
        assert!(db.get_cached_definition("new", "openai").unwrap().is_some());
        assert!(db.get_cached_definition("old", "openai").unwrap().is_none());
    }
}
