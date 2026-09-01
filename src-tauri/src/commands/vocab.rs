use crate::db::Database;
use crate::errors::AppError;
use crate::models::{VocabEntry, VocabNoteSnippet, VocabUpsertResult};
use crate::services::vocab_note::{
    heading_gloss, remove_from_split_files, resolve_note_path, sync_vocab_notes, NoteSyncKind,
};
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

fn short_definition_from_notes(notes: &[VocabNoteSnippet]) -> String {
    notes
        .first()
        .map(heading_gloss)
        .filter(|text| !text.is_empty())
        .unwrap_or_default()
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
        let (note_appended, note_error) = sync_notes_if_enabled(&db, &entry, &notes);
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
    BlockingExecutor::run(move || {
        db.delete_vocab_entry(&validated)?;
        if let Err(err) = remove_split_notes_if_enabled(&db, &validated) {
            log::warn!("Failed to remove vocab from split notes: {}", err);
        }
        Ok(())
    })
    .await
}

fn sync_notes_if_enabled(
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

    match sync_vocab_notes(
        &path,
        settings.vocab_note_split_files,
        &entry.word,
        entry.stars,
        &entry.category,
        notes,
    ) {
        Ok(NoteSyncKind::Appended) => {
            log::info!("Appended vocab note for '{}' to {:?}", entry.word, path);
            (true, None)
        }
        Ok(NoteSyncKind::Updated) => {
            log::info!("Updated vocab note for '{}' in {:?}", entry.word, path);
            (false, None)
        }
        Ok(NoteSyncKind::Skipped) => (false, None),
        Err(err) => {
            let message = format!("Failed to write vocab note: {}", err);
            log::warn!("{} ({})", message, path.display());
            (false, Some(message))
        }
    }
}

fn remove_split_notes_if_enabled(db: &Database, word: &str) -> Result<(), AppError> {
    let settings = db.get_settings()?;
    if !settings.vocab_note_enabled || !settings.vocab_note_split_files {
        return Ok(());
    }
    let Some(path) = settings.vocab_note_path.filter(|path| !path.trim().is_empty()) else {
        return Ok(());
    };
    let path = resolve_note_path(&path)?;
    remove_from_split_files(&path, word)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snippet(brief_gloss: &str, definition: &str) -> VocabNoteSnippet {
        VocabNoteSnippet {
            part_of_speech: "n.".into(),
            brief_gloss: brief_gloss.into(),
            definition: definition.into(),
            example: None,
        }
    }

    #[test]
    fn short_definition_uses_brief_gloss_not_full_definition() {
        let notes = [snippet(
            "微控制器",
            "微控制器是一种将处理器、存储器和输入输出接口集成在单个芯片上的小型计算机。",
        )];
        assert_eq!(short_definition_from_notes(&notes), "微控制器");
    }

    #[test]
    fn short_definition_falls_back_to_cjk_prefix() {
        let notes = [snippet(
            "",
            "微控制器是一种将处理器、存储器和输入输出接口集成在单个芯片上的小型计算机。",
        )];
        assert_eq!(short_definition_from_notes(&notes), "微控制器是一种将");
    }

    #[test]
    fn short_definition_truncates_overflow() {
        let notes = [snippet("一二三四五六七八九十十一十二十三", "ignored")];
        assert_eq!(
            short_definition_from_notes(&notes).chars().count(),
            16
        );
    }
}
