use crate::db::Database;
use crate::errors::AppError;
use crate::services::command_orchestrator::CommandOrchestrator;
use tauri::{AppHandle, State};

/// Search for a phrase definition using progressive loading (3 sections)
#[tauri::command]
pub async fn search_phrase_progressive(
    phrase: String,
    app: AppHandle,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<(), AppError> {
    log::info!("[Command] search_phrase_progressive '{}'", phrase);
    orchestrator.search_phrase_progressive(&phrase, &app, &db).await
}
