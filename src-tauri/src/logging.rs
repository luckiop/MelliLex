use log::{Level, LevelFilter};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::Wry;
use tauri_plugin_log::{Builder, RotationStrategy, Target, TargetKind};

const ENV_LOG_LEVEL: &str = "MELLILEX_LOG";

static DEV_LOG_ENABLED: AtomicBool = AtomicBool::new(false);
static ENV_LEVEL: OnceLock<LevelFilter> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub struct LogLevelConfig {
    level: LevelFilter,
}

impl LogLevelConfig {
    pub fn from_env() -> Self {
        let level = env::var(ENV_LOG_LEVEL)
            .ok()
            .and_then(|value| match value.to_uppercase().as_str() {
                "TRACE" => Some(LevelFilter::Trace),
                "DEBUG" => Some(LevelFilter::Debug),
                "INFO" => Some(LevelFilter::Info),
                "WARN" => Some(LevelFilter::Warn),
                "ERROR" => Some(LevelFilter::Error),
                _ => None,
            })
            .unwrap_or(LevelFilter::Info);
        Self { level }
    }

    pub fn level(&self) -> LevelFilter {
        self.level
    }
}

pub fn set_dev_log_enabled(enabled: bool) {
    let previous = DEV_LOG_ENABLED.swap(enabled, Ordering::Relaxed);
    if previous != enabled {
        log::info!(
            "Developer logging {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }
}

pub fn is_dev_log_enabled() -> bool {
    DEV_LOG_ENABLED.load(Ordering::Relaxed)
}

fn persist_to_file(metadata: &log::Metadata<'_>) -> bool {
    if is_dev_log_enabled() {
        return true;
    }
    let env_level = ENV_LEVEL.get().copied().unwrap_or(LevelFilter::Info);
    match env_level {
        LevelFilter::Off => false,
        LevelFilter::Error => metadata.level() <= Level::Error,
        LevelFilter::Warn => metadata.level() <= Level::Warn,
        LevelFilter::Info => metadata.level() <= Level::Info,
        LevelFilter::Debug => metadata.level() <= Level::Debug,
        LevelFilter::Trace => true,
    }
}

pub fn build_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    let config = LogLevelConfig::from_env();
    let _ = ENV_LEVEL.set(config.level());

    Builder::new()
        .max_file_size(5_000_000)
        .rotation_strategy(RotationStrategy::KeepAll)
        .level(LevelFilter::Trace)
        .targets([
            Target::new(TargetKind::Stdout).filter(|metadata| metadata.level() <= Level::Info),
            Target::new(TargetKind::LogDir {
                file_name: Some("mellilex".to_string()),
            })
            .filter(|metadata| persist_to_file(metadata)),
            Target::new(TargetKind::Webview).filter(|metadata| metadata.level() <= Level::Info),
        ])
        .build()
}
