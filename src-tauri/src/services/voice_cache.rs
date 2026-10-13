//! 0.13.0 spoken-audio cache. Map: `src/release/v0_13_0.ts`.
//!
//! One wav per normalized text and quantized volume, speed, and speaker.
//! A later slider value gets a new file. This module never deletes siblings.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::errors::AppError;
use crate::models::VoiceSettings;

const DIR_NAME: &str = "voice-cache";

pub fn dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(DIR_NAME)
}

/// Round to the slider steps so nearby floats share one recording.
pub fn quantize(settings: VoiceSettings) -> VoiceSettings {
    let settings = settings.clamped();
    VoiceSettings {
        volume: (settings.volume * 10.0).round() / 10.0,
        speed: (settings.speed * 100.0).round() / 100.0,
        speaker: settings.speaker,
    }
}

pub fn read(cache_dir: &Path, text: &str, voice: &VoiceSettings) -> Option<Vec<u8>> {
    let path = cache_dir.join(file_name(text, voice));
    let bytes = fs::read(path).ok()?;
    if bytes.len() > 44 && bytes.starts_with(b"RIFF") {
        Some(bytes)
    } else {
        None
    }
}

pub fn store(cache_dir: &Path, text: &str, voice: &VoiceSettings, wav: &[u8]) -> Result<(), AppError> {
    if text.trim().is_empty() || wav.len() <= 44 {
        return Ok(());
    }
    fs::create_dir_all(cache_dir)?;
    let path = cache_dir.join(file_name(text, voice));
    let temporary = path.with_extension("tmp");
    {
        let mut file = File::create(&temporary)?;
        file.write_all(wav)?;
    }
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    fs::rename(&temporary, &path)?;
    Ok(())
}

fn file_name(text: &str, voice: &VoiceSettings) -> String {
    let spoken = text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    let speed = (voice.speed * 100.0).round() as i32;
    let volume = (voice.volume * 10.0).round() as i32;
    format!(
        "{:016x}-s{speed}-v{volume}-k{}.wav",
        fnv64(spoken.as_bytes()),
        voice.speaker
    )
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn list_wavs(cache_dir: &Path) -> Vec<(PathBuf, SystemTime, u64)> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("wav") {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        files.push((path, modified, meta.len()));
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voice(volume: f32, speed: f32, speaker: u8) -> VoiceSettings {
        VoiceSettings { volume, speed, speaker }
    }

    fn wav(label: &str) -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend(std::iter::repeat(0).take(40));
        bytes.extend(label.as_bytes());
        bytes
    }

    #[test]
    fn a_second_read_returns_the_stored_recording() {
        let root = tempfile::tempdir().unwrap();
        let settings = voice(1.7, 0.9, 3);
        let bytes = wav("stored");
        store(root.path(), "Hello", &settings, &bytes).unwrap();
        assert_eq!(read(root.path(), "  hello  ", &settings).as_deref(), Some(bytes.as_slice()));
    }

    #[test]
    fn another_slider_value_keeps_the_previous_recording() {
        let root = tempfile::tempdir().unwrap();
        let quiet = voice(1.7, 0.9, 2);
        let fuller = voice(1.7, 0.9, 3);
        let quiet_wav = wav("quiet");
        let fuller_wav = wav("fuller");
        store(root.path(), "hello", &quiet, &quiet_wav).unwrap();
        store(root.path(), "hello", &fuller, &fuller_wav).unwrap();

        assert_eq!(list_wavs(root.path()).len(), 2);
        assert_eq!(read(root.path(), "hello", &quiet).as_deref(), Some(quiet_wav.as_slice()));
        assert_eq!(read(root.path(), "hello", &fuller).as_deref(), Some(fuller_wav.as_slice()));
    }
}
