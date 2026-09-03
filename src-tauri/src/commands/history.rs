use crate::db::Database;
use crate::errors::AppError;
use crate::models::WordHistory;
use crate::utils::blocking::BlockingExecutor;
use tauri::State;

#[tauri::command]
pub async fn get_history(
    limit: Option<i32>,
    db: State<'_, Database>,
) -> Result<Vec<WordHistory>, AppError> {
    let limit = limit.unwrap_or(50).min(2000).max(1);
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.get_history(limit)).await
}

#[tauri::command]
pub async fn clear_history(db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_history()).await
}

#[tauri::command]
pub async fn delete_history_item(id: String, db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.delete_history_item(&id)).await
}
