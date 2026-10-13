use std::path::{Path, PathBuf};

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

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|err| AppError::FileSystem(format!("App data directory is unavailable: {err}")))
}

fn exe_dir() -> Result<PathBuf, AppError> {
    let exe = std::env::current_exe()
        .map_err(|err| AppError::FileSystem(format!("App location is unavailable: {err}")))?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::FileSystem("App location is unavailable".into()))
}

#[tauri::command]
pub fn kokoro_status(app: AppHandle) -> Result<kokoro::KokoroStatus, AppError> {
    Ok(kokoro::inspect(&app_data_dir(&app)?, &exe_dir()?))
}

#[tauri::command]
pub async fn kokoro_download(
    app: AppHandle,
    state: State<'_, KokoroState>,
) -> Result<(), AppError> {
    let app_data = app_data_dir(&app)?;
    let exe = exe_dir()?;
    let emitter = app.clone();
    kokoro::install_voice(&app_data, &exe, |progress| {
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
    let app_data = app_data_dir(&app)?;
    let exe = exe_dir()?;
    let engine = state.inner().clone();
    let db = app.state::<crate::db::Database>().inner().clone();
    let wav = BlockingExecutor::run(move || {
        let voice = crate::services::voice_cache::quantize(db.get_settings()?.voice);
        let cache_dir = crate::services::voice_cache::dir(&app_data);
        if let Some(cached) = crate::services::voice_cache::read(&cache_dir, &text, &voice) {
            return Ok(cached);
        }
        let wav = kokoro::synthesize(
            &engine,
            &app_data,
            &exe,
            &text,
            voice.speed,
            i32::from(voice.speaker),
            voice.volume,
        )?;
        if let Err(err) = crate::services::voice_cache::store(&cache_dir, &text, &voice, &wav) {
            log::warn!("Spoken audio was not cached: {err}");
        }
        Ok(wav)
    })
    .await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(wav))
}
