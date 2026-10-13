use crate::db::Database;
use crate::errors::AppError;
use crate::services::storage::{self, StorageClearResult, StorageReport};
use crate::utils::blocking::BlockingExecutor;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn get_cache_stats(db: State<'_, Database>) -> Result<(usize, usize, usize, i64), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.get_cache_stats()).await
}

#[tauri::command]
pub async fn clear_all_cache(db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_all_cache()).await
}

#[tauri::command]
pub async fn clear_definition_cache(db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_definition_cache()).await
}

#[tauri::command]
pub async fn clear_exploration_cache(db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_exploration_cache()).await
}

#[tauri::command]
pub async fn storage_report(app: AppHandle, db: State<'_, Database>) -> Result<StorageReport, AppError> {
    let db = db.inner().clone();
    let app_data = app_data_dir(&app)?;
    let log_dir = log_dir(&app)?;
    BlockingExecutor::run(move || storage::report(&db, &app_data, &log_dir)).await
}

#[tauri::command]
pub async fn storage_clear(
    app: AppHandle,
    db: State<'_, Database>,
    kind: String,
    older_than_days: Option<u32>,
) -> Result<StorageClearResult, AppError> {
    let db = db.inner().clone();
    let app_data = app_data_dir(&app)?;
    let log_dir = log_dir(&app)?;
    BlockingExecutor::run(move || storage::clear(&db, &app_data, &log_dir, &kind, older_than_days)).await
}

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|err| AppError::FileSystem(format!("App data directory is unavailable: {err}")))
}

fn log_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    app.path()
        .app_log_dir()
        .map_err(|err| AppError::FileSystem(format!("Log directory is unavailable: {err}")))
}

#[tauri::command]
pub async fn clear_old_cache(db: State<'_, Database>, days: i64) -> Result<usize, AppError> {
    if days < 1 || days > 365 {
        return Err(AppError::validation("Days must be between 1 and 365"));
    }
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_old_cache(days)).await
}
