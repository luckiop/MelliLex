use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::errors::AppError;
use crate::services::kokoro::{self, KokoroState};
use crate::utils::blocking::BlockingExecutor;

#[derive(Clone, Serialize)]
struct DownloadProgress {
    progress: u8,
}

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|err| AppError::FileSystem(format!("App data directory is unavailable: {err}")))
}

#[tauri::command]
pub fn kokoro_status(app: AppHandle) -> Result<bool, AppError> {
    let dir = kokoro::model_dir(&app_data_dir(&app)?);
    Ok(kokoro::model_ready(&dir))
}

#[tauri::command]
pub async fn kokoro_download(
    app: AppHandle,
    state: State<'_, KokoroState>,
) -> Result<(), AppError> {
    let app_data = app_data_dir(&app)?;
    let emitter = app.clone();
    kokoro::download_model(&app_data, |progress| {
        let _ = emitter.emit(
            kokoro::DOWNLOAD_EVENT,
            DownloadProgress { progress },
        );
    })
    .await?;
    state.clear_engine();
    Ok(())
}

#[tauri::command]
pub async fn kokoro_reset(app: AppHandle, state: State<'_, KokoroState>) -> Result<(), AppError> {
    let dir = kokoro::model_dir(&app_data_dir(&app)?);
    kokoro::remove_model(&dir)?;
    state.clear_engine();
    Ok(())
}

#[tauri::command]
pub async fn kokoro_synthesize(
    app: AppHandle,
    state: State<'_, KokoroState>,
    text: String,
) -> Result<String, AppError> {
    let dir = kokoro::model_dir(&app_data_dir(&app)?);
    let engine = state.inner().clone();
    let wav = BlockingExecutor::run(move || kokoro::synthesize(&engine, &dir, &text)).await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(wav))
}
