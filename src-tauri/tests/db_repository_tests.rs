/// Integration tests for database repositories
/// Tests history, saved words, cache operations using temporary databases
mod common;

use common::create_test_db;
use mellilex_lib::models::*;

#[test]
fn test_history_basic_operations() {
    let (db, _temp_dir) = create_test_db();

    // Test adding to history
    let result = db.add_to_history("ephemeral", "openai");
    assert!(result.is_ok(), "Failed to add to history");

    // Test retrieving history with limit
    let history = db.get_history(100).expect("Failed to get history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].word, "ephemeral");
    assert_eq!(history[0].ai_provider, "openai");

    // Test adding multiple entries
    let _ = db.add_to_history("lament", "anthropic");
    let _ = db.add_to_history("serendipity", "gemini");
    let history = db.get_history(100).expect("Failed to get history");
    assert_eq!(history.len(), 3);

    // Test history limit
    let limited_history = db.get_history(2).expect("Failed to get limited history");
    assert_eq!(limited_history.len(), 2, "Should respect limit");
}

#[test]
fn test_cache_definition_operations() {
    let (db, _temp_dir) = create_test_db();

    // Create a sample definition using current model structure
    let definition = AiWordDefinition {
        word: "test".to_string(),
        phonetic: Some("/test/".to_string()),
        domain_tags: vec![],
        complexity: None,
        frequency: None,
        etymology: None,
        syllable_info: None,
        metrics: None,
        meanings: vec![AiMeaning {
            part_of_speech: "noun".to_string(),
            definitions: vec![Definition {
                text: "A procedure for critical evaluation".to_string(),
                examples: vec!["She passed the test".to_string()],
                contextual_examples: vec![],
                memory_tip: None,
                confidence: None,
            }],
            synonyms: vec![],
            antonyms: vec![],
            collocations: vec![],
        }],
        common_mistakes: vec![],
        contextual_usage: None,
        token_usage: None,
    };

    // Test caching definition
    let result = db.cache_definition("test", &definition, "openai");
    assert!(result.is_ok(), "Failed to cache definition");

    // Test retrieving cached definition
    let cached = db
        .get_cached_definition("test", "openai")
        .expect("Failed to get cached definition");
    assert!(cached.is_some(), "Cached definition should exist");
    let cached_def = cached.unwrap();
    assert_eq!(cached_def.word, "test");
    assert_eq!(cached_def.phonetic, Some("/test/".to_string()));
    assert_eq!(cached_def.meanings.len(), 1);

    // Test retrieving non-existent cache
    let cached = db
        .get_cached_definition("nonexistent", "openai")
        .expect("Failed to query cache");
    assert!(cached.is_none(), "Non-existent word should return None");

    // Test provider-specific caching
    let cached = db
        .get_cached_definition("test", "anthropic")
        .expect("Failed to query cache");
    assert!(cached.is_none(), "Different provider should not find cache");
}

#[test]
fn test_progressive_cache_operations() {
    let (db, _temp_dir) = create_test_db();

    // Create sample progressive data
    let section1 = WordSection1Header {
        word: "lament".to_string(),
        pronunciation: "/ləˈment/".to_string(),
        syllables: "la·ment".to_string(),
        origin: "Latin lamentum".to_string(),
        formality: FormalityInfo {
            level: "Neutral".to_string(),
            percentage: 50,
        },
        domains: vec![],
        tldr: "To express grief or regret".to_string(),
    };

    let section2 = WordSection2Meanings {
        meanings: vec![MeaningItem {
            number: 1,
            part_of_speech: "verb".to_string(),
            brief_gloss: String::new(),
            definition: "To express grief or regret".to_string(),
            memory_tip: "Think of 'lament' as expressing deep sadness".to_string(),
            examples: vec!["She lamented the loss of her friend".to_string()],
        }],
    };

    let section3 = WordSection3Related {
        synonyms: vec!["mourn".to_string(), "grieve".to_string()],
        antonyms: vec!["celebrate".to_string(), "rejoice".to_string()],
        collocations: vec![
            CollocationItem {
                phrase: "lament deeply".to_string(),
                example: "She lamented deeply over her mistake".to_string(),
            },
            CollocationItem {
                phrase: "lament over".to_string(),
                example: "He lamented over the lost opportunity".to_string(),
            },
        ],
    };

    let progressive_data = WordProgressiveData {
        section1: section1.clone(),
        section2: section2.clone(),
        mistakes: None,
        section3: section3.clone(),
    };

    // Test caching progressive data
    let result = db.cache_word_progressive("lament", &progressive_data, "openai");
    assert!(result.is_ok(), "Failed to cache progressive data");

    // Test retrieving progressive cache
    let cached = db
        .get_cached_word_progressive("lament", "openai")
        .expect("Failed to get cached progressive data");
    assert!(cached.is_some(), "Progressive cache should exist");

    let cached_data = cached.unwrap();
    assert_eq!(cached_data.section1.word, "lament");
    assert_eq!(cached_data.section2.meanings.len(), 1);
    assert_eq!(cached_data.section3.synonyms.len(), 2);

    // Test provider isolation
    let cached = db
        .get_cached_word_progressive("lament", "anthropic")
        .expect("Failed to query cache");
    assert!(cached.is_none(), "Different provider should not find cache");
}

#[test]
fn test_cache_clearing() {
    let (db, _temp_dir) = create_test_db();

    // Add some cached data using current model structure
    let definition = AiWordDefinition {
        word: "test".to_string(),
        phonetic: None,
        domain_tags: vec![],
        complexity: None,
        frequency: None,
        etymology: None,
        syllable_info: None,
        metrics: None,
        meanings: vec![],
        common_mistakes: vec![],
        contextual_usage: None,
        token_usage: None,
    };
    let _ = db.cache_definition("test", &definition, "openai");

    // Verify cache exists
    let cached = db
        .get_cached_definition("test", "openai")
        .expect("Failed to get cache");
    assert!(cached.is_some(), "Cache should exist before clearing");
}

#[test]
fn test_settings_persistence() {
    let (db, _temp_dir) = create_test_db();

    // Create test settings matching current model structure
    let settings = AppSettings {
        ai_provider: "openai".to_string(),
        open_ai_config: Some(OpenAiConfig {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
        }),
        deepseek_config: None,
        anthropic_config: None,
        gemini_config: None,
        ollama_config: None,
        theme: "dark".to_string(),
        export_settings: None,
        explanation_language: Some("Spanish".to_string()),
        ui_language: None,
        enable_global_lookup: false,
        typography_mode: "modern".to_string(),
        technical_query: false,
        technical_domain: default_technical_domain(),
        clipboard_click_search: true,
        vocab_note_enabled: false,
        vocab_note_path: None,
        vocab_note_split_files: false,
        dev_log_enabled: false,
        max_meanings_display: 5,
    };

    // Save settings
    let result = db.save_settings(&settings);
    assert!(result.is_ok(), "Failed to save settings");

    // Retrieve settings
    let loaded = db.get_settings().expect("Failed to get settings");
    assert_eq!(loaded.ai_provider, "openai");
    assert_eq!(loaded.explanation_language, Some("Spanish".to_string()));
    assert!(!loaded.enable_global_lookup);

    // Verify provider config is loaded
    assert!(loaded.open_ai_config.is_some());
    let openai = loaded.open_ai_config.unwrap();
    assert_eq!(openai.model, "gpt-4");
    assert_eq!(openai.api_key, "test-key");
}

#[test]
fn test_concurrent_operations() {
    use std::sync::Arc;
    use std::thread;

    let (db, _temp_dir) = create_test_db();
    let db = Arc::new(db);

    // Spawn multiple threads writing to history
    let mut handles = vec![];
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let word = format!("word{}", i);
            db_clone.add_to_history(&word, "openai").unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all entries were added
    let history = db.get_history(100).expect("Failed to get history");
    assert_eq!(history.len(), 5, "All concurrent writes should succeed");
}
