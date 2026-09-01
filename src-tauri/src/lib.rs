mod commands;
mod constants;
mod errors;
mod export;
mod global_lookup;
mod logging;
mod repositories;
mod security;
mod tray;
mod tray_events;
mod tray_menu;
mod utils;
mod validation;
mod window_controls;

pub mod db;
pub mod models;
pub mod services;

use commands::*;
use db::Database;
use services::command_orchestrator::CommandOrchestrator;
use tauri::Manager;
use std::panic;
use std::fs::OpenOptions;
use std::io::Write;

fn write_crash_log(message: &str) {
    // Write to a crash log file in a location we can always access
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let crash_log_dir = std::path::Path::new(&local_app_data).join("com.rahultr.mellilex").join("logs");
        let _ = std::fs::create_dir_all(&crash_log_dir);
        let crash_log_path = crash_log_dir.join("crash.log");
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&crash_log_path)
        {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] {}", timestamp, message);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set up panic hook to catch crashes
    panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        
        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };
        
        let crash_msg = format!("PANIC at {}: {}", location, msg);
        write_crash_log(&crash_msg);
        eprintln!("{}", crash_msg);
    }));

    write_crash_log("=== Application starting ===");
    write_crash_log("Step 1: Creating builder");
    log::info!("Starting AI Dictionary application");

    let mut builder = tauri::Builder::default();

    // Conditionally enable Tauri auto-updater
    // For Microsoft Store builds, disable updater (Store handles updates)
    // For direct downloads (GitHub), enable updater
    write_crash_log("Step 2: Checking store build");
    if !utils::platform::is_store_build() {
        write_crash_log("Step 2a: Adding updater plugin");
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
        log::info!("[Setup] Tauri auto-updater enabled (GitHub channel)");
    } else {
        log::info!("[Setup] Store build detected - Microsoft Store handles updates");
    }

    write_crash_log("Step 3: Adding plugins");
    write_crash_log("Step 13: Generating context");
    let context = tauri::generate_context!();
    write_crash_log("Step 14: Context generated, calling run()");
    
    let result = builder
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::SIZE,
                )
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .on_window_event(window_controls::handle_window_event)
        .plugin(tauri_plugin_opener::init())
        .plugin(logging::build_plugin())
        .setup(|app| {
            write_crash_log("Step 4: Setup callback started");
            log::info!("Starting AI Dictionary application");

            // Get app data directory in Tauri 2.x
            write_crash_log("Step 5: Getting app data dir");
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            log::info!("App data directory: {:?}", app_data_dir);
            write_crash_log(&format!("App data dir: {:?}", app_data_dir));

            write_crash_log("Step 6: Getting log dir");
            let log_dir = app
                .path()
                .app_log_dir()
                .expect("Failed to get app log directory");

            log::info!("Logs directory: {:?}", log_dir);
            std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");

            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            // Initialize resource path resolver for bundled assets
            write_crash_log("Step 6b: Initializing resource path resolver");
            let resource_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource directory");
            write_crash_log(&format!("Resource dir: {:?}", resource_dir));
            utils::resource_path::init(resource_dir.clone());
            log::info!("Resource path resolver initialized: {:?}", resource_dir);

            write_crash_log("Step 7: Initializing database");
            let db_path = app_data_dir.join("mellilex.db");
            log::info!("Database path: {:?}", db_path);

            let database = Database::new(db_path).map_err(|e| {
                let err_msg = format!("Failed to initialize database: {}", e);
                write_crash_log(&err_msg);
                log::error!("{}", err_msg);
                err_msg
            })?;

            if let Ok(settings) = database.get_settings() {
                crate::logging::set_dev_log_enabled(settings.dev_log_enabled);
            }

            log::info!("Database initialized successfully");
            write_crash_log("Step 8: Database initialized, managing state");
            app.manage(database);
            app.manage(CommandOrchestrator::default());

            // Apply platform-specific visual effects (with custom title bar)
            write_crash_log("Step 9: Getting main window");
            if let Some(window) = app.get_webview_window("main") {
                write_crash_log("Step 9a: Main window found, applying effects");
                #[cfg(target_os = "windows")]
                {
                    use window_vibrancy::apply_mica;

                    if let Err(e) = apply_mica(&window, None) {
                        write_crash_log(&format!("Failed to apply Mica: {}", e));
                        log::warn!("Failed to apply Mica effect: {}", e);
                    } else {
                        write_crash_log("Mica applied successfully");
                        log::info!("Windows 11 Mica effect applied successfully");
                    }
                }

                #[cfg(target_os = "macos")]
                {
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

                    if let Err(e) =
                        apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                    {
                        log::warn!("Failed to apply macOS vibrancy: {}", e);
                    } else {
                        log::info!("macOS vibrancy effect applied successfully");
                    }
                }

                #[cfg(target_os = "linux")]
                {
                    log::info!("Linux detected - window effects handled by compositor");
                }
            } else {
                write_crash_log("Step 9b: Main window NOT found!");
                log::warn!("Main window not found during setup");
            }

            write_crash_log("Step 10: Initializing tray");
            let app_handle = app.handle();
            tray::init(app_handle.clone()).map_err(|e| {
                let err_msg = format!("Failed to initialize tray: {}", e);
                write_crash_log(&err_msg);
                log::error!("{}", err_msg);
                err_msg
            })?;

            write_crash_log("Step 11: Initializing global lookup");
            let db_state = app_handle.state::<Database>();
            global_lookup::init(app_handle.clone(), &db_state).map_err(|e| {
                let err_msg = format!("Failed to initialize global lookup services: {}", e);
                write_crash_log(&err_msg);
                log::error!("{}", err_msg);
                err_msg
            })?;
            
            write_crash_log("Step 12: Setup complete!");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Search commands
            ping,
            check_spelling,
            get_word_variations,
            search_word_progressive,
            // Phrase commands
            search_phrase_progressive,
            // History commands
            get_history,
            clear_history,
            delete_history_item,
            // Vocab notebook
            get_vocab_entry,
            list_vocab_entries,
            upsert_vocab_entry,
            delete_vocab_entry,
            // Settings & provider commands
            get_settings,
            update_settings,
            update_ai_provider,
            detect_ollama,
            list_ollama_models,
            fetch_available_models,
            test_api_key,
            // Exploration feature commands
            generate_contextual_examples,
            generate_formality_analysis,
            generate_domain_exploration,
            generate_usage_patterns,
            generate_practice_exercises_only,
            generate_common_mistakes,
            get_cached_exploration_features,
            // Cache commands
            get_cache_stats,
            clear_all_cache,
            clear_definition_cache,
            clear_exploration_cache,
            clear_old_cache,
            // Export commands
            export_markdown_file,
            export_phrase_markdown_file,
            export_to_capacities,
            // Global lookup commands
            enable_global_lookup,
            disable_global_lookup,
            // System commands
            is_store_version,
            check_for_app_updates,
            get_log_dir,
            dev_log,
            log_client_error,
        ])
        .run(context);
    
    match result {
        Ok(_) => {
            write_crash_log("Application exited normally");
        }
        Err(e) => {
            let err_msg = format!("FATAL: Tauri run failed: {}", e);
            write_crash_log(&err_msg);
            eprintln!("{}", err_msg);
            panic!("{}", err_msg);
        }
    }
}
