use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig,
};

use crate::errors::AppError;

pub const MODEL_DIR_NAME: &str = "kokoro-int8-en-v0_19";
const MODEL_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-int8-en-v0_19.tar.bz2";
const ARCHIVE_NAME: &str = "kokoro-int8-en-v0_19.tar.bz2";
const MIN_MODEL_BYTES: u64 = 20_000_000;
const MIN_VOICES_BYTES: u64 = 100_000;
const MAX_TEXT_CHARS: usize = 400;
const SPEAKER_ID: i32 = 2;
const SPEAK_SPEED: f32 = 0.9;

pub const DOWNLOAD_EVENT: &str = "kokoro-download-progress";

#[derive(Clone, Default)]
pub struct KokoroState {
    engine: Arc<Mutex<Option<OfflineTts>>>,
}

impl KokoroState {
    pub fn clear_engine(&self) {
        if let Ok(mut engine) = self.engine.lock() {
            *engine = None;
        }
    }
}

pub fn model_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(MODEL_DIR_NAME)
}

pub fn model_ready(dir: &Path) -> bool {
    file_at_least(&dir.join("model.int8.onnx"), MIN_MODEL_BYTES)
        && file_at_least(&dir.join("voices.bin"), MIN_VOICES_BYTES)
        && file_nonempty(&dir.join("tokens.txt"))
        && dir.join("espeak-ng-data").join("phontab").is_file()
}

pub fn remove_model(dir: &Path) -> Result<(), AppError> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn synthesize(state: &KokoroState, dir: &Path, text: &str) -> Result<Vec<u8>, AppError> {
    let spoken = text.trim();
    if spoken.is_empty() {
        return Err(AppError::validation("Nothing to pronounce"));
    }
    if spoken.chars().count() > MAX_TEXT_CHARS {
        return Err(AppError::validation("Pronunciation text is too long"));
    }
    if !model_ready(dir) {
        return Err(AppError::not_found("Kokoro voice is not installed"));
    }

    let mut engine = state
        .engine
        .lock()
        .map_err(|_| AppError::lock_poisoned("kokoro"))?;
    if engine.is_none() {
        *engine = Some(load_engine(dir)?);
    }
    let tts = engine.as_ref().expect("engine was just loaded");
    let sid = if tts.num_speakers() > SPEAKER_ID {
        SPEAKER_ID
    } else {
        0
    };
    let config = GenerationConfig {
        sid,
        speed: SPEAK_SPEED,
        ..Default::default()
    };
    let audio = tts
        .generate_with_config(spoken, &config, Some(|_: &[f32], _: f32| true))
        .ok_or_else(|| AppError::Config("Kokoro synthesis failed".into()))?;
    let samples = audio.samples();
    if samples.is_empty() {
        return Err(AppError::Config("Kokoro returned empty audio".into()));
    }
    Ok(encode_wav_pcm16(samples, audio.sample_rate() as u32))
}

pub async fn download_model(
    app_data_dir: &Path,
    on_progress: impl Fn(u8),
) -> Result<(), AppError> {
    let dest = model_dir(app_data_dir);
    if model_ready(&dest) {
        on_progress(100);
        return Ok(());
    }

    fs::create_dir_all(app_data_dir)?;
    let archive_path = app_data_dir.join(ARCHIVE_NAME);
    let unpack_dir = app_data_dir.join("kokoro-unpack");
    if unpack_dir.exists() {
        fs::remove_dir_all(&unpack_dir)?;
    }
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }

    download_archive(&archive_path, &on_progress).await?;
    extract_archive(&archive_path, &unpack_dir)?;
    let root = find_model_root(&unpack_dir).ok_or_else(|| {
        AppError::FileSystem("Downloaded voice archive did not contain the Kokoro model".into())
    })?;
    fs::rename(&root, &dest)?;
    let _ = fs::remove_dir_all(&unpack_dir);
    let _ = fs::remove_file(&archive_path);

    if !model_ready(&dest) {
        let _ = fs::remove_dir_all(&dest);
        return Err(AppError::FileSystem(
            "Downloaded Kokoro voice is incomplete".into(),
        ));
    }
    on_progress(100);
    Ok(())
}

fn load_engine(dir: &Path) -> Result<OfflineTts, AppError> {
    let config = OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            kokoro: OfflineTtsKokoroModelConfig {
                model: Some(utf8_path(&dir.join("model.int8.onnx"))?),
                voices: Some(utf8_path(&dir.join("voices.bin"))?),
                tokens: Some(utf8_path(&dir.join("tokens.txt"))?),
                data_dir: Some(utf8_path(&dir.join("espeak-ng-data"))?),
                length_scale: 1.0,
                ..Default::default()
            },
            num_threads: 2,
            debug: false,
            ..Default::default()
        },
        ..Default::default()
    };
    OfflineTts::create(&config).ok_or_else(|| AppError::Config("Failed to load Kokoro".into()))
}

async fn download_archive(path: &Path, on_progress: &impl Fn(u8)) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(20))
        .timeout(std::time::Duration::from_secs(600))
        .user_agent("MelliLex")
        .build()?;
    let response = client.get(MODEL_URL).send().await?.error_for_status()?;
    let total = response.content_length().unwrap_or(0);
    let mut file = tokio::fs::File::create(path).await?;
    let mut received: u64 = 0;
    let mut last_progress = 0u8;
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        received += chunk.len() as u64;
        file.write_all(&chunk).await?;
        if total > 0 {
            let progress = ((received.saturating_mul(100)) / total).min(99) as u8;
            if progress != last_progress {
                last_progress = progress;
                on_progress(progress);
            }
        }
    }
    file.flush().await?;
    Ok(())
}

fn extract_archive(archive_path: &Path, dest: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dest)?;
    let file = File::open(archive_path)?;
    let decoder = bzip2::read::BzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest)
        .map_err(|err| AppError::FileSystem(format!("Failed to unpack voice archive: {err}")))?;
    Ok(())
}

fn find_model_root(unpack_dir: &Path) -> Option<PathBuf> {
    let direct = unpack_dir.join(MODEL_DIR_NAME);
    if direct.join("model.int8.onnx").is_file() {
        return Some(direct);
    }
    let entries = fs::read_dir(unpack_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("model.int8.onnx").is_file() {
            return Some(path);
        }
    }
    None
}

fn utf8_path(path: &Path) -> Result<String, AppError> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| AppError::FileSystem(format!("Path is not valid Unicode: {}", path.display())))
}

fn file_at_least(path: &Path, bytes: u64) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.len() >= bytes)
        .unwrap_or(false)
}

fn file_nonempty(path: &Path) -> bool {
    file_at_least(path, 1)
}

pub fn encode_wav_pcm16(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = samples.len() * 2;
    let mut bytes = Vec::with_capacity(44 + data_len);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_len as u32).to_le_bytes());
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let value = (clamped * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn touch(path: &Path, len: u64) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let file = File::create(path).unwrap();
        file.set_len(len).unwrap();
    }

    #[test]
    fn model_ready_accepts_a_complete_directory() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(MODEL_DIR_NAME);
        touch(&root.join("model.int8.onnx"), MIN_MODEL_BYTES);
        touch(&root.join("voices.bin"), MIN_VOICES_BYTES);
        touch(&root.join("tokens.txt"), 4);
        touch(&root.join("espeak-ng-data").join("phontab"), 8);
        assert!(model_ready(&root));
    }

    #[test]
    fn model_ready_rejects_a_short_model_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(&root.join("model.int8.onnx"), 32);
        touch(&root.join("voices.bin"), MIN_VOICES_BYTES);
        touch(&root.join("tokens.txt"), 4);
        touch(&root.join("espeak-ng-data").join("phontab"), 8);
        assert!(!model_ready(root));
    }

    #[test]
    fn extracts_model_files_from_a_bzip_archive() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join(ARCHIVE_NAME);
        write_archive(&archive_path);

        let unpack = dir.path().join("unpack");
        extract_archive(&archive_path, &unpack).unwrap();
        let root = find_model_root(&unpack).unwrap();
        assert_eq!(root.file_name().unwrap(), MODEL_DIR_NAME);
        assert!(root.join("model.int8.onnx").is_file());
        assert!(root.join("espeak-ng-data").join("phontab").is_file());
    }

    #[test]
    fn wav_header_describes_mono_pcm16() {
        let wav = encode_wav_pcm16(&[0.0, 1.0, -1.0], 24_000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 24_000);
        assert_eq!(i16::from_le_bytes(wav[44..46].try_into().unwrap()), 0);
        assert_eq!(i16::from_le_bytes(wav[46..48].try_into().unwrap()), i16::MAX);
        assert_eq!(i16::from_le_bytes(wav[48..50].try_into().unwrap()), -i16::MAX);
    }

    fn write_archive(path: &Path) {
        let stored = tempfile::tempdir().unwrap();
        let root = stored.path().join(MODEL_DIR_NAME);
        fs::create_dir_all(root.join("espeak-ng-data")).unwrap();
        fs::write(root.join("model.int8.onnx"), b"model").unwrap();
        fs::write(root.join("voices.bin"), b"voices").unwrap();
        fs::write(root.join("tokens.txt"), b"tokens").unwrap();
        fs::write(root.join("espeak-ng-data").join("phontab"), b"phontab").unwrap();

        let tar_path = stored.path().join("model.tar");
        let tar_file = File::create(&tar_path).unwrap();
        let mut builder = tar::Builder::new(tar_file);
        builder
            .append_dir_all(MODEL_DIR_NAME, &root)
            .unwrap();
        builder.finish().unwrap();

        let tar_bytes = fs::read(tar_path).unwrap();
        let archive = File::create(path).unwrap();
        let mut encoder = bzip2::write::BzEncoder::new(archive, bzip2::Compression::fast());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap();
    }
}
