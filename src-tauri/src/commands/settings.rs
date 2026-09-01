use crate::db::Database;
use crate::errors::AppError;
use crate::models::AppSettings;
use crate::services::ai_provider::AiModel;
use crate::services::command_orchestrator::CommandOrchestrator;
use crate::services::vocab_note::{ensure_note_file, resolve_note_path};
use crate::utils::blocking::BlockingExecutor;
use tauri::State;

#[tauri::command]
pub async fn get_settings(db: State<'_, Database>) -> Result<AppSettings, AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.get_settings()).await
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    db: State<'_, Database>,
) -> Result<(), AppError> {
    let db = db.inner().clone();
    let dev_log_enabled = settings.dev_log_enabled;
    BlockingExecutor::run(move || {
        if settings.vocab_note_enabled {
            if let Some(path) = settings
                .vocab_note_path
                .as_deref()
                .filter(|path| !path.trim().is_empty())
            {
                let resolved = resolve_note_path(path)?;
                ensure_note_file(&resolved)?;
            }
        }
        db.save_settings(&settings)
    })
    .await?;
    crate::logging::set_dev_log_enabled(dev_log_enabled);
    Ok(())
}

#[tauri::command]
pub async fn update_ai_provider(
    provider: String,
    config: serde_json::Value,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<(), AppError> {
    log::info!("Updating AI provider to: {}", provider);
    // NOTE: Do not log config values - they contain sensitive API keys
    orchestrator
        .update_ai_provider(&provider, config, &db)
        .map_err(|e| {
            log::error!("Failed to update provider {}: {}", provider, e);
            e
        })
}

#[tauri::command]
pub async fn detect_ollama(
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<bool, AppError> {
    orchestrator.detect_ollama(&db).await
}

#[tauri::command]
pub async fn list_ollama_models(
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<String>, AppError> {
    orchestrator.list_ollama_models(&db).await
}

#[tauri::command]
pub async fn fetch_available_models(
    provider: String,
    api_key: String,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<AiModel>, AppError> {
    log::info!("Fetching models for provider: {}", provider);
    orchestrator
        .fetch_available_models(&provider, &api_key, &db)
        .await
}

#[tauri::command]
pub async fn test_api_key(
    provider: String,
    api_key: String,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<bool, AppError> {
    log::info!("Testing API key for provider: {}", provider);
    orchestrator.test_api_key(&provider, &api_key, &db).await
}
