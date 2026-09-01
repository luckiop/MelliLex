use crate::errors::AppError;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn is_store_version() -> bool {
    crate::utils::platform::is_store_build()
}

#[tauri::command]
pub fn get_log_dir(app: AppHandle) -> Result<String, AppError> {
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|err| AppError::FileSystem(format!("Log directory is not available: {err}")))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn dev_log(message: String) {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return;
    }
    let preview: String = trimmed.chars().take(2000).collect();
    log::debug!("[ui] {preview}");
}

#[tauri::command]
pub fn log_client_error(message: String) {
    let preview: String = message.trim().chars().take(4000).collect();
    if !preview.is_empty() {
        log::error!("[ui] {preview}");
    }
}

#[tauri::command]
pub async fn check_for_app_updates(app: AppHandle) -> Result<bool, AppError> {
    let result = if crate::utils::platform::is_store_build() {
        // Store builds don't use Tauri updater (Microsoft Store handles updates)
        log::info!("[Command] Update check skipped - Store version (managed by Microsoft Store)");
        Ok(false)
    } else {
        #[cfg(debug_assertions)]
        {
            log::info!("[Command] Update check disabled in development mode");
            drop(app);
            Ok(false)
        }

        #[cfg(not(debug_assertions))]
        {
            use tauri_plugin_updater::UpdaterExt;

            log::info!("[Command] Checking for application updates (GitHub channel)");

            let updater = app.updater_builder().build().map_err(|e| {
                log::error!("Failed to build updater: {}", e);
                AppError::Config(format!("Failed to build updater: {}", e))
            })?;

            match updater.check().await {
                Ok(Some(update)) => {
                    log::info!(
                        "Update available: {} -> {}",
                        update.current_version,
                        update.version
                    );
                    Ok(true)
                }
                Ok(None) => {
                    log::info!("App is up to date");
                    Ok(false)
                }
                Err(e) => {
                    log::error!("Update check failed: {}", e);
                    Err(AppError::Config(format!("Update check failed: {}", e)))
                }
            }
        }
    };

    result
}
