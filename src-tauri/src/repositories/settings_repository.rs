use crate::errors::AppError;
use crate::models::AppSettings;
use crate::utils::sync;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SettingsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SettingsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn load(&self) -> Result<AppSettings, AppError> {
        let conn = sync::lock(&self.conn, "settings connection")?;
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = 'app_settings'")
            .map_err(AppError::from)?;

        let result = stmt.query_row([], |row| {
            let settings_str: String = row.get(0)?;
            Ok(settings_str)
        });

        match result {
            Ok(settings_str) => {
                let settings: AppSettings = serde_json::from_str(&settings_str).map_err(|e| {
                    AppError::Parse(format!("Failed to deserialize settings: {}", e))
                })?;
                Ok(settings)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings {
                ai_provider: "anthropic".to_string(),
                open_ai_config: None,
                anthropic_config: None,
                gemini_config: None,
                ollama_config: None,
                theme: "light".to_string(),
                export_settings: None,
                explanation_language: Some("English".to_string()),
                ui_language: None, // Will be auto-detected on frontend first run
                enable_global_lookup: true,
                typography_mode: "classic".to_string(),
                technical_query: false,
                vocab_note_enabled: false,
                vocab_note_path: None,
            }),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "settings connection")?;
        let settings_json = serde_json::to_string(settings)?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('app_settings', ?1)",
            params![settings_json],
        )
        .map_err(AppError::from)?;

        Ok(())
    }
}
