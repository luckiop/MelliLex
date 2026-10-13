use std::ffi::CString;
use std::fs::{self, File};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use libloading::{Library, Symbol};
use serde::Serialize;

use crate::errors::AppError;

pub const MODEL_DIR_NAME: &str = "kokoro-int8-en-v0_19";
const MODEL_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-int8-en-v0_19.tar.bz2";
const ARCHIVE_NAME: &str = "kokoro-int8-en-v0_19.tar.bz2";
const RUNTIME_DIR_NAME: &str = "kokoro-runtime";
const RUNTIME_ARCHIVE_NAME: &str = "sherpa-onnx-v1.13.8-win-x64-shared-MT-Release-lib.tar.bz2";
const RUNTIME_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.8/sherpa-onnx-v1.13.8-win-x64-shared-MT-Release-lib.tar.bz2";
const PLUGIN_FILE: &str = "mellilex_kokoro.dll";
const MIN_MODEL_BYTES: u64 = 20_000_000;
const MIN_VOICES_BYTES: u64 = 100_000;
const MIN_PLUGIN_BYTES: u64 = 50_000;
const MAX_TEXT_CHARS: usize = 400;

const RUNTIME_FILES: &[(&str, u64)] = &[
    ("sherpa-onnx-c-api.dll", 1_000_000),
    ("onnxruntime.dll", 5_000_000),
    ("onnxruntime_providers_shared.dll", 50_000),
];

pub const DOWNLOAD_EVENT: &str = "kokoro-download-progress";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KokoroStatus {
    pub plugin_ready: bool,
    pub runtime_ready: bool,
    pub model_ready: bool,
}

impl KokoroStatus {
    pub fn complete(&self) -> bool {
        self.plugin_ready && self.runtime_ready && self.model_ready
    }
}

#[derive(Clone)]
pub struct KokoroState {
    library: Arc<Mutex<Option<Library>>>,
}

impl Default for KokoroState {
    fn default() -> Self {
        Self {
            library: Arc::new(Mutex::new(None)),
        }
    }
}

impl KokoroState {
    pub fn clear_engine(&self) {
        let Ok(mut library) = self.library.lock() else {
            return;
        };
        if let Some(loaded) = library.as_ref() {
            unsafe {
                if let Ok(reset) = loaded.get::<unsafe extern "C" fn()>(b"mellilex_kokoro_reset\0") {
                    reset();
                }
            }
        }
        *library = None;
    }
}

pub fn model_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(MODEL_DIR_NAME)
}

pub fn runtime_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(RUNTIME_DIR_NAME)
}

pub fn inspect(app_data_dir: &Path, exe_dir: &Path) -> KokoroStatus {
    KokoroStatus {
        plugin_ready: plugin_source(exe_dir).is_some(),
        runtime_ready: runtime_ready(exe_dir) || runtime_ready(&runtime_dir(app_data_dir)),
        model_ready: model_ready(&model_dir(app_data_dir)),
    }
}

fn plugin_source(exe_dir: &Path) -> Option<PathBuf> {
    [exe_dir.join(PLUGIN_FILE), exe_dir.join("resources").join(PLUGIN_FILE)]
        .into_iter()
        .find(|path| file_at_least(path, MIN_PLUGIN_BYTES))
}

pub fn model_ready(dir: &Path) -> bool {
    file_at_least(&dir.join("model.int8.onnx"), MIN_MODEL_BYTES)
        && file_at_least(&dir.join("voices.bin"), MIN_VOICES_BYTES)
        && file_nonempty(&dir.join("tokens.txt"))
        && dir.join("espeak-ng-data").join("phontab").is_file()
}

pub fn runtime_ready(dir: &Path) -> bool {
    RUNTIME_FILES.iter().all(|(name, bytes)| file_at_least(&dir.join(name), *bytes))
}

pub fn remove_model(dir: &Path) -> Result<(), AppError> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn synthesize(
    state: &KokoroState,
    app_data_dir: &Path,
    exe_dir: &Path,
    text: &str,
    speed: f32,
    speaker: i32,
    gain: f32,
) -> Result<Vec<u8>, AppError> {
    let spoken = text.trim();
    if spoken.is_empty() {
        return Err(AppError::validation("Nothing to pronounce"));
    }
    if spoken.chars().count() > MAX_TEXT_CHARS {
        return Err(AppError::validation("Pronunciation text is too long"));
    }
    let status = inspect(app_data_dir, exe_dir);
    if !status.plugin_ready {
        return Err(AppError::not_found("Kokoro speech component is not in this app"));
    }
    if !status.runtime_ready {
        return Err(AppError::not_found("Kokoro speech libraries are not installed"));
    }
    if !status.model_ready {
        return Err(AppError::not_found("Kokoro voice is not installed"));
    }

    let load_dir = prepare_load_dir(app_data_dir, exe_dir)?;
    let mut library = state
        .library
        .lock()
        .map_err(|_| AppError::lock_poisoned("kokoro"))?;
    if library.is_none() {
        *library = Some(load_plugin(&load_dir)?);
    }
    let loaded = library.as_ref().expect("plugin was just loaded");
    let (samples, rate) = unsafe { call_synthesize(loaded, &model_dir(app_data_dir), spoken, speed, speaker)? };
    Ok(encode_wav_pcm16(&apply_gain(&samples, gain), rate as u32))
}

pub async fn install_voice(
    app_data_dir: &Path,
    exe_dir: &Path,
    on_progress: impl Fn(u8),
) -> Result<(), AppError> {
    let status = inspect(app_data_dir, exe_dir);
    if !status.plugin_ready {
        return Err(AppError::not_found(
            "This app build does not include the speech component",
        ));
    }

    let needs_runtime = !status.runtime_ready;
    let needs_model = !status.model_ready;
    if !needs_runtime && !needs_model {
        on_progress(100);
        return Ok(());
    }

    if needs_runtime && needs_model {
        download_runtime(app_data_dir, |progress| on_progress(scale(progress, 0, 40))).await?;
        download_model(app_data_dir, |progress| on_progress(scale(progress, 40, 100))).await?;
    } else if needs_runtime {
        download_runtime(app_data_dir, &on_progress).await?;
    } else {
        download_model(app_data_dir, &on_progress).await?;
    }
    on_progress(100);
    Ok(())
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

    download_archive(MODEL_URL, &archive_path, &on_progress).await?;
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

fn prepare_load_dir(app_data_dir: &Path, exe_dir: &Path) -> Result<PathBuf, AppError> {
    let plugin = plugin_source(exe_dir)
        .ok_or_else(|| AppError::not_found("Kokoro speech component is not in this app"))?;
    let load_dir = if runtime_ready(exe_dir) {
        exe_dir.to_path_buf()
    } else if runtime_ready(&runtime_dir(app_data_dir)) {
        runtime_dir(app_data_dir)
    } else {
        return Err(AppError::not_found("Kokoro speech libraries are not installed"));
    };
    let dest = load_dir.join(PLUGIN_FILE);
    if dest != plugin && !plugin_is_current(&plugin, &dest) {
        fs::copy(&plugin, &dest)?;
    }
    Ok(load_dir)
}

fn plugin_is_current(source: &Path, dest: &Path) -> bool {
    let Ok(source_meta) = fs::metadata(source) else {
        return false;
    };
    let Ok(dest_meta) = fs::metadata(dest) else {
        return false;
    };
    if dest_meta.len() < MIN_PLUGIN_BYTES || dest_meta.len() != source_meta.len() {
        return false;
    }
    match (source_meta.modified(), dest_meta.modified()) {
        (Ok(source_time), Ok(dest_time)) => dest_time >= source_time,
        _ => true,
    }
}

fn load_plugin(dir: &Path) -> Result<Library, AppError> {
    let path = std::path::absolute(dir.join(PLUGIN_FILE))?;
    unsafe {
        libloading::os::windows::Library::load_with_flags(
            &path,
            libloading::os::windows::LOAD_WITH_ALTERED_SEARCH_PATH,
        )
    }
    .map(Library::from)
    .map_err(|err| AppError::Config(format!("Failed to load the speech component: {err}")))
}

type SynthesizeFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    f32,
    i32,
    *mut *mut f32,
    *mut i32,
    *mut i32,
) -> i32;
type FreeFn = unsafe extern "C" fn(*mut f32, i32);
type ErrorFn = unsafe extern "C" fn(*mut u8, i32) -> i32;

unsafe fn call_synthesize(
    library: &Library,
    model_dir: &Path,
    text: &str,
    speed: f32,
    speaker: i32,
) -> Result<(Vec<f32>, i32), AppError> {
    let synthesize: Symbol<SynthesizeFn> = library
        .get(b"mellilex_kokoro_synthesize\0")
        .map_err(|err| AppError::Config(format!("Speech component cannot synthesize: {err}")))?;
    let free_samples: Symbol<FreeFn> = library
        .get(b"mellilex_kokoro_free\0")
        .map_err(|err| AppError::Config(format!("Speech component cannot release audio: {err}")))?;
    let last_error: Result<Symbol<ErrorFn>, _> = library.get(b"mellilex_kokoro_last_error\0");

    let model = CString::new(utf8_path(model_dir)?)
        .map_err(|_| AppError::validation("Voice path is invalid"))?;
    let spoken = CString::new(text).map_err(|_| AppError::validation("Pronunciation text is invalid"))?;
    let mut samples_ptr: *mut f32 = std::ptr::null_mut();
    let mut len = 0i32;
    let mut rate = 0i32;
    let code = synthesize(
        model.as_ptr(),
        spoken.as_ptr(),
        speed,
        speaker,
        &mut samples_ptr,
        &mut len,
        &mut rate,
    );
    if code != 0 || samples_ptr.is_null() || len <= 0 {
        let message = read_plugin_error(last_error.as_ref().ok());
        if !samples_ptr.is_null() {
            free_samples(samples_ptr, len);
        }
        return Err(AppError::Config(message));
    }
    let samples = std::slice::from_raw_parts(samples_ptr, len as usize).to_vec();
    free_samples(samples_ptr, len);
    Ok((samples, rate))
}

fn read_plugin_error(symbol: Option<&Symbol<ErrorFn>>) -> String {
    let Some(symbol) = symbol else {
        return "Kokoro synthesis failed".into();
    };
    let mut buf = vec![0u8; 512];
    let len = unsafe { symbol(buf.as_mut_ptr(), buf.len() as i32) };
    if len <= 0 {
        return "Kokoro synthesis failed".into();
    }
    String::from_utf8_lossy(&buf[..len as usize]).into_owned()
}

async fn download_runtime(app_data_dir: &Path, on_progress: impl Fn(u8)) -> Result<(), AppError> {
    let dest = runtime_dir(app_data_dir);
    if runtime_ready(&dest) {
        on_progress(100);
        return Ok(());
    }

    fs::create_dir_all(app_data_dir)?;
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    let archive_path = app_data_dir.join(RUNTIME_ARCHIVE_NAME);
    let unpack_dir = app_data_dir.join("kokoro-runtime-unpack");
    if unpack_dir.exists() {
        fs::remove_dir_all(&unpack_dir)?;
    }

    download_archive(RUNTIME_URL, &archive_path, &on_progress).await?;
    extract_archive(&archive_path, &unpack_dir)?;
    fs::create_dir_all(&dest)?;
    copy_runtime_files(&unpack_dir, &dest)?;
    let _ = fs::remove_dir_all(&unpack_dir);
    let _ = fs::remove_file(&archive_path);

    if !runtime_ready(&dest) {
        let _ = fs::remove_dir_all(&dest);
        return Err(AppError::FileSystem(
            "Downloaded speech libraries are incomplete".into(),
        ));
    }
    on_progress(100);
    Ok(())
}

fn copy_runtime_files(unpack_dir: &Path, dest: &Path) -> Result<(), AppError> {
    let mut copied = 0usize;
    for path in walk_files(unpack_dir) {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if RUNTIME_FILES.iter().any(|(expected, _)| *expected == name) {
            fs::copy(&path, dest.join(name))?;
            copied += 1;
        }
    }
    if copied < RUNTIME_FILES.len() {
        return Err(AppError::FileSystem(
            "Downloaded speech libraries are incomplete".into(),
        ));
    }
    Ok(())
}

fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files
}

fn scale(progress: u8, start: u8, end: u8) -> u8 {
    let span = u16::from(end.saturating_sub(start));
    start.saturating_add(((u16::from(progress) * span) / 100) as u8)
}

async fn download_archive(url: &str, path: &Path, on_progress: &impl Fn(u8)) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(20))
        .timeout(std::time::Duration::from_secs(600))
        .user_agent("MelliLex")
        .build()?;
    let response = client.get(url).send().await?.error_for_status()?;
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

pub fn apply_gain(samples: &[f32], gain: f32) -> Vec<f32> {
    let gain = if gain.is_finite() { gain.clamp(0.8, 2.2) } else { 1.7 };
    samples
        .iter()
        .map(|sample| (sample * gain).clamp(-1.0, 1.0))
        .collect()
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

    #[test]
    fn gain_makes_a_quiet_sample_louder_without_exceeding_full_scale() {
        let louder = apply_gain(&[0.2], 1.7);
        assert!((louder[0] - 0.34).abs() < 0.001);
        assert_eq!(apply_gain(&[0.8], 2.2)[0], 1.0);
        assert_eq!(apply_gain(&[-0.8], 2.2)[0], -1.0);
    }

    #[test]
    fn inspect_accepts_a_bundled_plugin_and_an_app_data_runtime() {
        let root = tempfile::tempdir().unwrap();
        let exe_dir = root.path().join("app");
        let app_data = root.path().join("data");
        touch(&exe_dir.join("resources").join(PLUGIN_FILE), MIN_PLUGIN_BYTES);
        for (name, bytes) in RUNTIME_FILES {
            touch(&runtime_dir(&app_data).join(name), *bytes);
        }
        let model = model_dir(&app_data);
        touch(&model.join("model.int8.onnx"), MIN_MODEL_BYTES);
        touch(&model.join("voices.bin"), MIN_VOICES_BYTES);
        touch(&model.join("tokens.txt"), 4);
        touch(&model.join("espeak-ng-data").join("phontab"), 8);

        let status = inspect(&app_data, &exe_dir);
        assert!(status.plugin_ready);
        assert!(status.runtime_ready);
        assert!(status.model_ready);
        assert!(status.complete());
    }

    #[test]
    fn inspect_reports_missing_libraries_without_touching_the_model() {
        let root = tempfile::tempdir().unwrap();
        let exe_dir = root.path().join("app");
        let app_data = root.path().join("data");
        touch(&exe_dir.join(PLUGIN_FILE), MIN_PLUGIN_BYTES);
        let model = model_dir(&app_data);
        touch(&model.join("model.int8.onnx"), MIN_MODEL_BYTES);
        touch(&model.join("voices.bin"), MIN_VOICES_BYTES);
        touch(&model.join("tokens.txt"), 4);
        touch(&model.join("espeak-ng-data").join("phontab"), 8);

        let status = inspect(&app_data, &exe_dir);
        assert!(status.plugin_ready);
        assert!(!status.runtime_ready);
        assert!(status.model_ready);
        assert!(model_ready(&model));
    }

    #[test]
    fn replaces_a_stale_plugin_already_sitting_beside_the_speech_libraries() {
        let root = tempfile::tempdir().unwrap();
        let exe_dir = root.path().join("app");
        let app_data = root.path().join("data");
        let runtime = runtime_dir(&app_data);
        for (name, bytes) in RUNTIME_FILES {
            touch(&runtime.join(name), *bytes);
        }
        let stale = runtime.join(PLUGIN_FILE);
        touch(&stale, MIN_PLUGIN_BYTES);
        fs::write(&stale, vec![0x11; MIN_PLUGIN_BYTES as usize]).unwrap();

        let bundled = exe_dir.join("resources").join(PLUGIN_FILE);
        touch(&bundled, MIN_PLUGIN_BYTES + 8);
        let fresh = vec![0x22; (MIN_PLUGIN_BYTES + 8) as usize];
        fs::write(&bundled, &fresh).unwrap();

        let load_dir = prepare_load_dir(&app_data, &exe_dir).unwrap();
        let copied = fs::read(runtime.join(PLUGIN_FILE)).unwrap();
        assert_eq!(load_dir, runtime);
        assert_eq!(copied.len(), fresh.len());
        assert_eq!(copied[0], 0x22);
    }

    #[test]
    fn copies_runtime_libraries_out_of_a_nested_archive() {
        let root = tempfile::tempdir().unwrap();
        let unpack = root.path().join("unpack").join("sherpa-onnx").join("lib");
        let dest = root.path().join("runtime");
        fs::create_dir_all(&dest).unwrap();
        for (name, bytes) in RUNTIME_FILES {
            touch(&unpack.join(name), *bytes);
        }
        touch(&unpack.join("sherpa-onnx-cxx-api.dll"), 200_000);

        copy_runtime_files(&root.path().join("unpack"), &dest).unwrap();
        assert!(runtime_ready(&dest));
        assert!(!dest.join("sherpa-onnx-cxx-api.dll").exists());
    }

    #[test]
    fn loads_the_plugin_from_its_own_directory() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let plugin_dir = manifest_dir.join("target/kokoro-plugin/debug");
        let deps_dir = manifest_dir.join("target/debug/deps");
        for name in [
            PLUGIN_FILE,
            "sherpa-onnx-c-api.dll",
            "onnxruntime.dll",
            "onnxruntime_providers_shared.dll",
        ] {
            let _ = fs::remove_file(deps_dir.join(name));
        }

        let root = tempfile::tempdir().unwrap();
        let dest = root.path().join("runtime");
        fs::create_dir_all(&dest).unwrap();
        for name in [
            PLUGIN_FILE,
            "sherpa-onnx-c-api.dll",
            "onnxruntime.dll",
            "onnxruntime_providers_shared.dll",
        ] {
            fs::copy(plugin_dir.join(name), dest.join(name)).unwrap();
        }

        let old_path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", r"C:\Windows\System32");
        let loaded = load_plugin(&dest);
        std::env::set_var("PATH", old_path);
        loaded.expect("speech component loads beside its libraries");
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
