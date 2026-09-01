use crate::db::Database;
use crate::errors::AppError;
use crate::models::{VocabEntry, VocabNoteSnippet};
use crate::services::vocab_note::{append_vocab_note, validate_note_path};
use crate::utils::blocking::BlockingExecutor;
use crate::validation;
use tauri::State;

fn normalize_category(category: &str) -> Result<&str, AppError> {
    match category.trim() {
        "daily" | "technical" => Ok(category.trim()),
        _ => Err(AppError::validation(
            "Vocab category must be daily or technical",
        )),
    }
}

fn normalize_stars(stars: i32) -> Result<i32, AppError> {
    if stars == 1 || stars == 2 {
        Ok(stars)
    } else {
        Err(AppError::validation("Vocab stars must be 1 or 2"))
    }
}

#[tauri::command]
pub async fn get_vocab_entry(
    word: String,
    db: State<'_, Database>,
) -> Result<Option<VocabEntry>, AppError> {
    let validated = validation::validate_word_query(&word)?;
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.get_vocab_entry(&validated)).await
}

#[tauri::command]
pub async fn list_vocab_entries(
    stars: Option<i32>,
    category: Option<String>,
    db: State<'_, Database>,
) -> Result<Vec<VocabEntry>, AppError> {
    let stars = match stars {
        Some(value) => Some(normalize_stars(value)?),
        None => None,
    };
    let category = match category.as_deref() {
        Some(value) => Some(normalize_category(value)?.to_string()),
        None => None,
    };
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.list_vocab_entries(stars, category.as_deref())).await
}

#[tauri::command]
pub async fn upsert_vocab_entry(
    word: String,
    stars: i32,
    category: String,
    notes: Option<Vec<VocabNoteSnippet>>,
    db: State<'_, Database>,
) -> Result<VocabEntry, AppError> {
    let validated = validation::validate_word_query(&word)?;
    let stars = normalize_stars(stars)?;
    let category = normalize_category(&category)?.to_string();
    let notes = notes.unwrap_or_default();
    let db = db.inner().clone();

    BlockingExecutor::run(move || {
        let (entry, inserted) = db.upsert_vocab_entry(&validated, stars, &category)?;
        if inserted {
            maybe_append_note(&db, &entry, &notes);
        }
        Ok(entry)
    })
    .await
}

#[tauri::command]
pub async fn delete_vocab_entry(word: String, db: State<'_, Database>) -> Result<(), AppError> {
    let validated = validation::validate_word_query(&word)?;
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.delete_vocab_entry(&validated)).await
}

fn maybe_append_note(db: &Database, entry: &VocabEntry, notes: &[VocabNoteSnippet]) {
    let settings = match db.get_settings() {
        Ok(settings) => settings,
        Err(err) => {
            log::warn!("Skipping vocab note: failed to load settings: {}", err);
            return;
        }
    };

    if !settings.vocab_note_enabled {
        return;
    }

    let Some(path) = settings.vocab_note_path.filter(|path| !path.trim().is_empty()) else {
        log::warn!("Vocab notes enabled but no path is set");
        return;
    };

    let path = match validate_note_path(&path) {
        Ok(path) => path,
        Err(err) => {
            log::warn!("Skipping vocab note: {}", err);
            return;
        }
    };

    if let Err(err) = append_vocab_note(&path, &entry.word, entry.stars, &entry.category, notes) {
        log::warn!("Failed to append vocab note for '{}': {}", entry.word, err);
    }
}
