//! 0.13.0 Kokoro plugin. Map: `src/release/v0_13_0.ts`.
//!
//! This cdylib is loaded only after the sherpa libraries are present, so the
//! main executable can start without them. `speed` and `sid` are arguments of
//! `mellilex_kokoro_synthesize`. Loudness is applied by the host before the
//! WAV is written; this library does not change sample amplitude.

use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig,
};

const SPEAK_SPEED: f32 = 0.9;

static ENGINE: Mutex<Option<OfflineTts>> = Mutex::new(None);
static LAST_ERROR: Mutex<String> = Mutex::new(String::new());
static BUFFERS: Mutex<Option<HashMap<usize, Vec<f32>>>> = Mutex::new(None);

fn set_error(message: impl Into<String>) {
    if let Ok(mut slot) = LAST_ERROR.lock() {
        *slot = message.into();
    }
}

#[no_mangle]
pub extern "C" fn mellilex_kokoro_reset() {
    if let Ok(mut engine) = ENGINE.lock() {
        *engine = None;
    }
}

#[no_mangle]
pub extern "C" fn mellilex_kokoro_last_error(buf: *mut u8, capacity: i32) -> i32 {
    if buf.is_null() || capacity <= 0 {
        return 0;
    }
    let message = LAST_ERROR.lock().map(|slot| slot.clone()).unwrap_or_default();
    let bytes = message.as_bytes();
    let cap = capacity as usize;
    let copy_len = bytes.len().min(cap.saturating_sub(1));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
        *buf.add(copy_len) = 0;
    }
    copy_len as i32
}

#[no_mangle]
pub unsafe extern "C" fn mellilex_kokoro_free(samples: *mut f32, _len: i32) {
    if samples.is_null() {
        return;
    }
    if let Ok(mut buffers) = BUFFERS.lock() {
        if let Some(buffers) = buffers.as_mut() {
            buffers.remove(&(samples as usize));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mellilex_kokoro_synthesize(
    model_dir: *const c_char,
    text: *const c_char,
    speed: f32,
    sid: i32,
    out_samples: *mut *mut f32,
    out_len: *mut i32,
    out_rate: *mut i32,
) -> i32 {
    if out_samples.is_null() || out_len.is_null() || out_rate.is_null() {
        set_error("Invalid speech output");
        return 1;
    }
    *out_samples = std::ptr::null_mut();
    *out_len = 0;
    *out_rate = 0;

    let result = std::panic::catch_unwind(|| synthesize(model_dir, text, speed, sid));
    match result {
        Ok(Ok((samples, rate))) => match hold_samples(samples) {
            Ok(ptr) => {
                *out_len = ptr.1;
                *out_rate = rate;
                *out_samples = ptr.0;
                0
            }
            Err(message) => {
                set_error(message);
                1
            }
        }
        Ok(Err(message)) => {
            set_error(message);
            1
        }
        Err(_) => {
            set_error("Speech engine failed unexpectedly");
            1
        }
    }
}

fn synthesize(
    model_dir: *const c_char,
    text: *const c_char,
    speed: f32,
    sid: i32,
) -> Result<(Vec<f32>, i32), String> {
    let model_dir = c_string(model_dir, "Voice directory is invalid")?;
    let text = c_string(text, "Pronunciation text is invalid")?;
    let spoken = text.trim();
    if spoken.is_empty() {
        return Err("Nothing to pronounce".into());
    }

    let mut engine = ENGINE.lock().map_err(|_| "Speech engine is unavailable".to_string())?;
    if engine.is_none() {
        *engine = Some(load_engine(&model_dir)?);
    }
    let tts = engine.as_ref().expect("engine was just loaded");
    let speed = if speed.is_finite() { speed.clamp(0.7, 1.3) } else { SPEAK_SPEED };
    let requested = sid.max(0);
    let sid = if tts.num_speakers() > requested { requested } else { 0 };
    let config = GenerationConfig {
        sid,
        speed,
        ..Default::default()
    };
    let audio = tts
        .generate_with_config(spoken, &config, Some(|_: &[f32], _: f32| true))
        .ok_or_else(|| "Kokoro synthesis failed".to_string())?;
    let samples = audio.samples();
    if samples.is_empty() {
        return Err("Kokoro returned empty audio".into());
    }
    Ok((samples.to_vec(), audio.sample_rate()))
}

fn load_engine(dir: &str) -> Result<OfflineTts, String> {
    let root = std::path::Path::new(dir);
    let config = OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            kokoro: OfflineTtsKokoroModelConfig {
                model: Some(join(root, "model.int8.onnx")?),
                voices: Some(join(root, "voices.bin")?),
                tokens: Some(join(root, "tokens.txt")?),
                data_dir: Some(join(root, "espeak-ng-data")?),
                length_scale: 1.0,
                ..Default::default()
            },
            num_threads: 2,
            debug: false,
            ..Default::default()
        },
        ..Default::default()
    };
    OfflineTts::create(&config).ok_or_else(|| "Failed to load Kokoro".to_string())
}

fn join(root: &std::path::Path, name: &str) -> Result<String, String> {
    root.join(name)
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| format!("Path is not valid Unicode: {}", root.join(name).display()))
}

fn hold_samples(samples: Vec<f32>) -> Result<(*mut f32, i32), String> {
    let mut buffers = BUFFERS
        .lock()
        .map_err(|_| "Speech engine is unavailable".to_string())?;
    let buffers = buffers.get_or_insert_with(HashMap::new);
    let len = samples.len() as i32;
    let ptr = samples.as_ptr() as *mut f32;
    buffers.insert(ptr as usize, samples);
    Ok((ptr, len))
}

fn c_string(ptr: *const c_char, message: &str) -> Result<String, String> {
    if ptr.is_null() {
        return Err(message.into());
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(str::to_string)
        .map_err(|_| message.to_string())
}
