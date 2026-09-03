use crate::errors::AppError;
use crate::models::AppSettings;
use crate::utils::sync;
use chrono::Utc;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use rusqlite::{params, Connection};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

pub const SECRET_OPENAI_API_KEY: &str = "secret:openai_api_key";
pub const SECRET_DEEPSEEK_API_KEY: &str = "secret:deepseek_api_key";
pub const SECRET_ANTHROPIC_API_KEY: &str = "secret:anthropic_api_key";
pub const SECRET_GEMINI_API_KEY: &str = "secret:gemini_api_key";
pub const SECRET_CAPACITIES_TOKEN: &str = "secret:capacities_api_token";
const SECRET_NONCE_LEN: usize = 12;

#[derive(Clone)]
pub struct SecretStore {
    cipher: SecretCipher,
    conn: Arc<Mutex<Connection>>,
}

impl SecretStore {
    pub fn new(conn: Arc<Mutex<Connection>>, key_path: &Path) -> Result<Self, AppError> {
        let cipher = SecretCipher::new(key_path)?;
        Ok(Self { cipher, conn })
    }

    pub fn hydrate_provider_secrets(&self, settings: &mut AppSettings) -> Result<(), AppError> {
        if let Some(cfg) = settings.open_ai_config.as_mut() {
            if let Some(secret) = self.load_secret(SECRET_OPENAI_API_KEY)? {
                cfg.api_key = secret;
            }
        }

        if let Some(cfg) = settings.deepseek_config.as_mut() {
            if let Some(secret) = self.load_secret(SECRET_DEEPSEEK_API_KEY)? {
                cfg.api_key = secret;
            }
        }

        if let Some(cfg) = settings.anthropic_config.as_mut() {
            if let Some(secret) = self.load_secret(SECRET_ANTHROPIC_API_KEY)? {
                cfg.api_key = secret;
            }
        }

        if let Some(cfg) = settings.gemini_config.as_mut() {
            if let Some(secret) = self.load_secret(SECRET_GEMINI_API_KEY)? {
                cfg.api_key = secret;
            }
        }

        if let Some(export) = settings.export_settings.as_mut() {
            if let Some(capacities) = export.capacities.as_mut() {
                if let Some(secret) = self.load_secret(SECRET_CAPACITIES_TOKEN)? {
                    capacities.api_token = secret;
                }
            }
        }

        Ok(())
    }

    pub fn persist_provider_secrets(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.store_secret(
            SECRET_OPENAI_API_KEY,
            settings
                .open_ai_config
                .as_ref()
                .map(|cfg| cfg.api_key.as_str()),
        )?;

        self.store_secret(
            SECRET_DEEPSEEK_API_KEY,
            settings
                .deepseek_config
                .as_ref()
                .map(|cfg| cfg.api_key.as_str()),
        )?;

        self.store_secret(
            SECRET_ANTHROPIC_API_KEY,
            settings
                .anthropic_config
                .as_ref()
                .map(|cfg| cfg.api_key.as_str()),
        )?;

        self.store_secret(
            SECRET_GEMINI_API_KEY,
            settings
                .gemini_config
                .as_ref()
                .map(|cfg| cfg.api_key.as_str()),
        )?;

        let capacities_token = settings
            .export_settings
            .as_ref()
            .and_then(|cfg| cfg.capacities.as_ref())
            .map(|capacities| capacities.api_token.as_str());
        self.store_secret(SECRET_CAPACITIES_TOKEN, capacities_token)?;

        Ok(())
    }

    pub fn sanitize_settings(settings: &AppSettings) -> AppSettings {
        let mut sanitized = settings.clone();

        if let Some(cfg) = sanitized.open_ai_config.as_mut() {
            cfg.api_key.clear();
        }

        if let Some(cfg) = sanitized.deepseek_config.as_mut() {
            cfg.api_key.clear();
        }

        if let Some(cfg) = sanitized.anthropic_config.as_mut() {
            cfg.api_key.clear();
        }

        if let Some(cfg) = sanitized.gemini_config.as_mut() {
            cfg.api_key.clear();
        }

        if let Some(export) = sanitized.export_settings.as_mut() {
            if let Some(capacities) = export.capacities.as_mut() {
                capacities.api_token.clear();
            }
        }

        sanitized
    }

    fn store_secret(&self, key: &str, value: Option<&str>) -> Result<(), AppError> {
        let trimmed = value.and_then(|v| {
            let val = v.trim();
            if val.is_empty() {
                None
            } else {
                Some(v)
            }
        });

        let conn = sync::lock(&self.conn, "secret store connection")?;

        if let Some(secret_value) = trimmed {
            let plaintext = Zeroizing::new(secret_value.as_bytes().to_vec());
            let (ciphertext, nonce) = self.cipher.encrypt(&plaintext)?;
            conn.execute(
                "INSERT OR REPLACE INTO secure_settings (key, value, nonce, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![
                    key,
                    ciphertext,
                    nonce.to_vec(),
                    Utc::now().timestamp_millis()
                ],
            )?;
        } else {
            conn.execute("DELETE FROM secure_settings WHERE key = ?1", params![key])?;
        }

        Ok(())
    }

    fn load_secret(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = sync::lock(&self.conn, "secret store connection")?;
        let mut stmt = conn.prepare("SELECT value, nonce FROM secure_settings WHERE key = ?1")?;

        let result = stmt.query_row(params![key], |row| {
            let value: Vec<u8> = row.get(0)?;
            let nonce: Vec<u8> = row.get(1)?;
            Ok((value, nonce))
        });

        match result {
            Ok((value, nonce)) => {
                let decrypted = self.cipher.decrypt(&nonce, &value)?;
                let secret = String::from_utf8(decrypted.to_vec())
                    .map_err(|e| AppError::secret(format!("Invalid UTF-8 secret: {}", e)))?;
                Ok(Some(secret))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }
}

#[derive(Clone)]
struct SecretCipher {
    key: LessSafeKey,
    rand: SystemRandom,
}

impl SecretCipher {
    fn new(key_path: &Path) -> Result<Self, AppError> {
        let rand = SystemRandom::new();
        let master_key = Self::load_or_create_key(key_path, &rand)?;
        let unbound = UnboundKey::new(&AES_256_GCM, master_key.as_slice())
            .map_err(|_| AppError::secret("Invalid master key length"))?;

        Ok(Self {
            key: LessSafeKey::new(unbound),
            rand,
        })
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; SECRET_NONCE_LEN]), AppError> {
        let mut nonce = [0u8; SECRET_NONCE_LEN];
        self.rand
            .fill(&mut nonce)
            .map_err(|_| AppError::secret("Failed to generate nonce"))?;

        let nonce_val = Nonce::assume_unique_for_key(nonce);
        let mut buffer = plaintext.to_vec();
        self.key
            .seal_in_place_append_tag(nonce_val, Aad::empty(), &mut buffer)
            .map_err(|_| AppError::secret("Failed to encrypt secret"))?;

        Ok((buffer, nonce))
    }

    fn decrypt(&self, nonce: &[u8], ciphertext: &[u8]) -> Result<Zeroizing<Vec<u8>>, AppError> {
        if nonce.len() != SECRET_NONCE_LEN {
            return Err(AppError::secret("Invalid nonce length"));
        }

        let mut buffer = ciphertext.to_vec();
        let mut nonce_arr = [0u8; SECRET_NONCE_LEN];
        nonce_arr.copy_from_slice(nonce);
        let plaintext = self
            .key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce_arr),
                Aad::empty(),
                &mut buffer,
            )
            .map_err(|_| AppError::secret("Failed to decrypt secret"))?;

        Ok(Zeroizing::new(plaintext.to_vec()))
    }

    fn load_or_create_key(
        path: &Path,
        rand: &SystemRandom,
    ) -> Result<Zeroizing<Vec<u8>>, AppError> {
        if path.exists() {
            let data = fs::read(path)?;
            if data.len() != 32 {
                return Err(AppError::secret("Master key must be 32 bytes"));
            }
            return Ok(Zeroizing::new(data));
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut key = Zeroizing::new(vec![0u8; 32]);
        rand.fill(&mut key[..])
            .map_err(|_| AppError::secret("Failed to generate master key"))?;

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)?;
            file.write_all(key.as_slice())?;
        }

        #[cfg(not(unix))]
        {
            fs::write(path, key.as_slice())?;
        }

        Ok(key)
    }
}
