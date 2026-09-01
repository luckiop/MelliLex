use crate::db::Database;
use crate::errors::AppError;
use crate::models::SpellCheckResponse;
use crate::services::command_orchestrator::CommandOrchestrator;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub async fn check_spelling(
    word: String,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<SpellCheckResponse, AppError> {
    orchestrator.check_spelling(&word).await
}

#[tauri::command]
pub async fn get_word_variations(
    word: String,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<String>, AppError> {
    orchestrator.get_word_variations(&word).await
}

#[tauri::command]
pub async fn search_word_progressive(
    word: String,
    app: AppHandle,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<(), AppError> {
    log::info!("[Command] search_word_progressive '{}'", word);
    orchestrator.search_word_progressive(&word, &app, &db).await
}
