use crate::db::Database;
use crate::errors::AppError;
use crate::models::{VocabEntry, VocabNoteSnippet, VocabUpsertResult};
use crate::services::vocab_note::{append_vocab_note, resolve_note_path};
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

const SHORT_DEFINITION_MAX_CHARS: usize = 180;

fn short_definition_from_notes(notes: &[VocabNoteSnippet]) -> String {
    let text = notes
        .first()
        .map(|note| note.definition.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_default();
    truncate_short_definition(&text)
}

fn truncate_short_definition(text: &str) -> String {
    let char_count = text.chars().count();
    if char_count <= SHORT_DEFINITION_MAX_CHARS {
        return text.to_string();
    }
    format!(
        "{}…",
        text.chars().take(SHORT_DEFINITION_MAX_CHARS).collect::<String>()
    )
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
) -> Result<VocabUpsertResult, AppError> {
    let validated = validation::validate_word_query(&word)?;
    let stars = normalize_stars(stars)?;
    let category = normalize_category(&category)?.to_string();
    let notes = notes.unwrap_or_default();
    let short_definition = short_definition_from_notes(&notes);
    let db = db.inner().clone();

    BlockingExecutor::run(move || {
        log::info!(
            "Vocab upsert '{}' stars={} category={}",
            validated,
            stars,
            category
        );
        let (entry, inserted) =
            db.upsert_vocab_entry(&validated, stars, &category, &short_definition)?;
        log::info!(
            "Vocab upsert '{}' inserted={} stars={}",
            entry.word,
            inserted,
            entry.stars
        );
        let (note_appended, note_error) = if inserted {
            append_note_if_enabled(&db, &entry, &notes)
        } else {
            (false, None)
        };
        Ok(VocabUpsertResult {
            entry,
            note_appended,
            note_error,
        })
    })
    .await
}

#[tauri::command]
pub async fn delete_vocab_entry(word: String, db: State<'_, Database>) -> Result<(), AppError> {
    let validated = validation::validate_word_query(&word)?;
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.delete_vocab_entry(&validated)).await
}

fn append_note_if_enabled(
    db: &Database,
    entry: &VocabEntry,
    notes: &[VocabNoteSnippet],
) -> (bool, Option<String>) {
    let settings = match db.get_settings() {
        Ok(settings) => settings,
        Err(err) => {
            log::warn!("Skipping vocab note: failed to load settings: {}", err);
            return (false, Some(err.to_string()));
        }
    };

    if !settings.vocab_note_enabled {
        return (false, None);
    }

    let Some(path) = settings.vocab_note_path.filter(|path| !path.trim().is_empty()) else {
        let message = "Vocab notes are enabled but no file path is set".to_string();
        log::warn!("{}", message);
        return (false, Some(message));
    };

    let path = match resolve_note_path(&path) {
        Ok(path) => path,
        Err(err) => {
            log::warn!("Skipping vocab note: {}", err);
            return (false, Some(err.to_string()));
        }
    };

    match append_vocab_note(&path, &entry.word, entry.stars, &entry.category, notes) {
        Ok(()) => {
            log::info!("Appended vocab note for '{}' to {:?}", entry.word, path);
            (true, None)
        }
        Err(err) => {
            let message = format!("Failed to write vocab note: {}", err);
            log::warn!("{} ({})", message, path.display());
            (false, Some(message))
        }
    }
}
