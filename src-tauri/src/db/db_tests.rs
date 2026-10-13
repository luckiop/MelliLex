use super::*;
use crate::models::{
    AiMeaning, CapacitiesSettings, Definition, ExportSettings, OpenAiConfig, OllamaConfig,
};
use crate::security::secret_store::{SECRET_CAPACITIES_TOKEN, SECRET_OPENAI_API_KEY};
use rusqlite::{params, Connection};
use std::collections::HashSet;
use tempfile::TempDir;

fn create_db() -> (Database, TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("test.db");
    let db = Database::new(path).expect("Failed to create database");
    (db, dir)
}

fn sample_definition() -> AiWordDefinition {
    AiWordDefinition {
        word: "sample".to_string(),
        phonetic: Some("/ˈsam.pəl/".to_string()),
        domain_tags: vec![],
        complexity: Some("Basic".to_string()),
        frequency: Some("Common".to_string()),
        etymology: None,
        syllable_info: None,
        metrics: None,
        meanings: vec![AiMeaning {
            part_of_speech: "noun".to_string(),
            definitions: vec![Definition {
                text: "a thing characteristic of its kind".to_string(),
                examples: vec!["She brought a sample for testing.".to_string()],
                contextual_examples: vec![],
                memory_tip: None,
                confidence: None,
            }],
            synonyms: vec!["example".to_string()],
            antonyms: vec![],
            collocations: vec![],
        }],
        common_mistakes: vec![],
        contextual_usage: None,
        token_usage: None,
    }
}

fn sample_settings_payload() -> AppSettings {
    AppSettings {
        ai_provider: "openai".to_string(),
        open_ai_config: Some(OpenAiConfig {
            api_key: "test-key".to_string(),
            model: "gpt-4o-mini".to_string(),
        }),
        deepseek_config: None,
        anthropic_config: None,
        gemini_config: None,
        ollama_config: Some(OllamaConfig {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama3".to_string(),
        }),
        theme: "dark".to_string(),
        export_settings: Some(ExportSettings {
            include_exploration: false,
            capacities: Some(CapacitiesSettings {
                api_token: "secret-token".to_string(),
                space_id: "space-123".to_string(),
                default_tags: vec!["vocabulary".into(), "ai".into()],
                no_timestamp: true,
            }),
        }),
        explanation_language: Some("English".to_string()),
        ui_language: Some("en-US".to_string()),
        enable_global_lookup: true,
        typography_mode: "classic".to_string(),
        technical_query: false,
        technical_domain: crate::models::default_technical_domain(),
        clipboard_click_search: true,
        vocab_note_enabled: false,
        vocab_note_path: None,
        vocab_note_split_files: false,
        dev_log_enabled: false,
        max_meanings_display: 5,
        voice: crate::models::VoiceSettings::default(),
    }
}

#[test]
fn history_crud_operations() {
    let (db, _dir) = create_db();

    let first = db.add_to_history("alpha", "mock").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let second = db.add_to_history("beta", "mock").unwrap();

    let history = db.get_history(10).unwrap();
    assert_eq!(history.len(), 2);
    let words: HashSet<_> = history.iter().map(|h| h.word.as_str()).collect();
    assert!(words.contains("alpha"));
    assert!(words.contains("beta"));

    db.delete_history_item(&first.id).unwrap();
    let history = db.get_history(10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].id, second.id);

    db.clear_history().unwrap();
    let history = db.get_history(10).unwrap();
    assert!(history.is_empty());

    let err = db.delete_history_item("missing-id").unwrap_err();
    assert!(
        err.to_string().contains("History item not found"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn cache_round_trip() {
    let (db, _dir) = create_db();
    let definition = sample_definition();

    db.cache_definition("lexicon", &definition, "openai").unwrap();

    let cached = db.get_cached_definition("lexicon", "openai").unwrap();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().word, "sample");

    let missing = db.get_cached_definition("unknown", "openai").unwrap();
    assert!(missing.is_none());
}

#[test]
fn settings_persistence() {
    let (db, _dir) = create_db();

    let default_settings = db.get_settings().unwrap();
    assert_eq!(default_settings.ai_provider, "anthropic");
    assert_eq!(default_settings.theme, "light");
    assert!(default_settings.export_settings.is_none());

    let new_settings = sample_settings_payload();

    db.save_settings(&new_settings).unwrap();

    let loaded = db.get_settings().unwrap();
    assert_eq!(loaded.ai_provider, "openai");
    assert_eq!(loaded.theme, "dark");
    let openai_cfg = loaded.open_ai_config.clone().unwrap();
    assert_eq!(openai_cfg.model, "gpt-4o-mini");
    assert_eq!(openai_cfg.api_key, "test-key");
    assert_eq!(loaded.ollama_config.unwrap().model, "llama3");
    let capacities = loaded
        .export_settings
        .and_then(|cfg| cfg.capacities)
        .expect("capacities config missing");
    assert_eq!(capacities.space_id, "space-123");
    assert!(capacities.no_timestamp);
}

#[test]
fn migrates_deepseek_alias_from_openai_slot() {
    let (db, _dir) = create_db();
    let mut settings = sample_settings_payload();
    settings.open_ai_config = Some(OpenAiConfig {
        api_key: "sk-deepseek".to_string(),
        model: "deepseek-v4-flash".to_string(),
    });
    db.save_settings(&settings).unwrap();

    let loaded = db.get_settings().unwrap();
    assert_eq!(loaded.ai_provider, "deepseek");
    assert!(loaded.open_ai_config.is_none());
    let deepseek = loaded.deepseek_config.expect("deepseek config");
    assert_eq!(deepseek.model, "deepseek-v4-flash");
    assert_eq!(deepseek.api_key, "sk-deepseek");
}

#[test]
fn secrets_are_stored_encrypted() {
    let (db, dir) = create_db();
    let settings = sample_settings_payload();
    db.save_settings(&settings).unwrap();

    let conn = Connection::open(dir.path().join("test.db")).unwrap();
    let stored_json: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'app_settings'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(!stored_json.contains("test-key"));
    assert!(!stored_json.contains("secret-token"));

    let openai_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM secure_settings WHERE key = ?1",
            params![SECRET_OPENAI_API_KEY],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(openai_count, 1);

    let capacities_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM secure_settings WHERE key = ?1",
            params![SECRET_CAPACITIES_TOKEN],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(capacities_count, 1);

    let loaded = db.get_settings().unwrap();
    let openai_cfg = loaded.open_ai_config.unwrap();
    assert_eq!(openai_cfg.api_key, "test-key");
    let capacities = loaded
        .export_settings
        .and_then(|cfg| cfg.capacities)
        .expect("capacities missing");
    assert_eq!(capacities.api_token, "secret-token");
}

#[test]
fn vocab_upsert_cycles_and_filters() {
    let (db, _dir) = create_db();

    let (first, inserted) = db
        .upsert_vocab_entry("Gain", 1, "technical", "loop gain")
        .unwrap();
    assert!(inserted);
    assert_eq!(first.word, "Gain");
    assert_eq!(first.stars, 1);
    assert_eq!(first.category, "technical");
    assert_eq!(first.short_definition, "loop gain");

    let loaded = db.get_vocab_entry("gain").unwrap().expect("entry");
    assert_eq!(loaded.stars, 1);

    let (second, inserted_again) = db.upsert_vocab_entry("gain", 2, "daily", "").unwrap();
    assert!(!inserted_again);
    assert_eq!(second.stars, 2);
    assert_eq!(second.category, "daily");
    assert_eq!(second.added_at, first.added_at);
    assert_eq!(second.short_definition, "loop gain");

    db.upsert_vocab_entry("loop", 1, "technical", "closed path").unwrap();

    let technical = db.list_vocab_entries(None, Some("technical")).unwrap();
    assert_eq!(technical.len(), 1);
    assert_eq!(technical[0].word, "loop");

    let two_star = db.list_vocab_entries(Some(2), None).unwrap();
    assert_eq!(two_star.len(), 1);
    assert_eq!(two_star[0].word, "gain");

    db.delete_vocab_entry("GAIN").unwrap();
    assert!(db.get_vocab_entry("gain").unwrap().is_none());
}
