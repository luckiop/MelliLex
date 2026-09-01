#[cfg(test)]
mod progressive_updates_tests {
    use crate::constants::providers;
    use crate::db::Database;
    use crate::models::{WordSection1Header, WordSection2Meanings, WordSection3Related};
    use crate::services::command_orchestrator::CommandOrchestrator;
    use crate::services::orchestration::TestEmitter;
    use crate::services::provider_registry::ProviderRegistry;

    #[tokio::test]
    async fn test_progressive_sections_emit_in_order_from_cache() {
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);
        let db = create_test_db();

        // Cache data
        setup_test_cache(&db);
        let cached = crate::models::WordProgressiveData {
            section1: WordSection1Header {
                word: "cached".to_string(),
                pronunciation: "/ˈkæʃt/".to_string(),
                syllables: "cached".to_string(),
                origin: "cache".to_string(),
                formality: crate::models::FormalityInfo {
                    level: "Neutral".to_string(),
                    percentage: 50,
                },
                domains: vec![],
                tldr: "Test TL;DR".to_string(),
            },
            section2: WordSection2Meanings {
                meanings: vec![crate::models::MeaningItem {
                    number: 1,
                    part_of_speech: "verb".to_string(),
                    brief_gloss: String::new(),
                    definition: "Stored for quick retrieval".to_string(),
                    memory_tip: "Think of a treasure cache".to_string(),
                    examples: vec!["The data is cached".to_string()],
                }],
            },
            mistakes: None,
            section3: WordSection3Related {
                synonyms: vec!["stored".to_string()],
                antonyms: vec![],
                collocations: vec![],
            },
        };

        db.cache_word_progressive("cached", &cached, "openai")
            .unwrap();

        // Now search with test emitter
        let emitter = TestEmitter::new();
        let result = orchestrator
            .search_word_progressive_with_emitter("cached", &db, &emitter)
            .await;

        assert!(result.is_ok());

        // Verify all sections were emitted
        assert!(emitter.get_section1().is_some());
        assert!(emitter.get_section2().is_some());
        assert!(emitter.get_section3().is_some());

        // Verify emission order
        let order = emitter.get_emission_order();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "section1");
        assert_eq!(order[1], "section2");
        assert_eq!(order[2], "section3");
    }

    #[tokio::test]
    async fn test_progressive_uses_fallback_provider_cache_before_live_generation() {
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);
        let db = create_test_db();

        // Configure a provider that is not cached and not configured. If fallback cache
        // lookup regresses, this path would attempt live generation and fail.
        let mut settings = db.get_settings().expect("Should load default settings");
        settings.ai_provider = providers::ANTHROPIC.to_string();
        settings.anthropic_config = None;
        db.save_settings(&settings)
            .expect("Should save provider settings");

        let cached = crate::models::WordProgressiveData {
            section1: WordSection1Header {
                word: "fallback-cache".to_string(),
                pronunciation: "/ˈfɔːlbæk kæʃ/".to_string(),
                syllables: "fall·back cache".to_string(),
                origin: "test fixture".to_string(),
                formality: crate::models::FormalityInfo {
                    level: "Neutral".to_string(),
                    percentage: 50,
                },
                domains: vec![],
                tldr: "Uses cached fallback provider data".to_string(),
            },
            section2: WordSection2Meanings {
                meanings: vec![crate::models::MeaningItem {
                    number: 1,
                    part_of_speech: "noun".to_string(),
                    brief_gloss: String::new(),
                    definition: "Data served from fallback provider cache".to_string(),
                    memory_tip: "Fallback avoids unnecessary live generation".to_string(),
                    examples: vec!["The app served fallback cached sections".to_string()],
                }],
            },
            mistakes: None,
            section3: WordSection3Related {
                synonyms: vec!["backup".to_string()],
                antonyms: vec!["miss".to_string()],
                collocations: vec![],
            },
        };

        db.cache_word_progressive("fallback-cache", &cached, providers::OPENAI)
            .expect("Should cache fallback provider data");

        let emitter = TestEmitter::new();
        let result = orchestrator
            .search_word_progressive_with_emitter("fallback-cache", &db, &emitter)
            .await;

        assert!(
            result.is_ok(),
            "Search should succeed by serving fallback provider cache"
        );

        let order = emitter.get_emission_order();
        assert_eq!(order, vec!["section1", "section2", "section3"]);
        assert_eq!(
            emitter
                .get_section1()
                .expect("Section1 should be emitted")
                .word,
            "fallback-cache"
        );
    }

    #[tokio::test]
    async fn test_progressive_sections_contain_correct_data_from_cache() {
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);
        let db = create_test_db();

        // Cache data
        let cached = crate::models::WordProgressiveData {
            section1: WordSection1Header {
                word: "progressive".to_string(),
                pronunciation: "/prəˈɡresɪv/".to_string(),
                syllables: "pro·gres·sive".to_string(),
                origin: "Latin progressus".to_string(),
                formality: crate::models::FormalityInfo {
                    level: "Formal".to_string(),
                    percentage: 70,
                },
                domains: vec!["Technology".to_string()],
                tldr: "Happening in stages".to_string(),
            },
            section2: WordSection2Meanings {
                meanings: vec![crate::models::MeaningItem {
                    number: 1,
                    part_of_speech: "adjective".to_string(),
                    brief_gloss: String::new(),
                    definition: "Happening gradually".to_string(),
                    memory_tip: "Progress happens progressively".to_string(),
                    examples: vec!["Progressive loading".to_string()],
                }],
            },
            mistakes: None,
            section3: WordSection3Related {
                synonyms: vec!["gradual".to_string(), "incremental".to_string()],
                antonyms: vec!["sudden".to_string()],
                collocations: vec![crate::models::CollocationItem {
                    phrase: "progressive enhancement".to_string(),
                    example: "Use progressive enhancement for better UX".to_string(),
                }],
            },
        };

        db.cache_word_progressive("progressive", &cached, "openai")
            .unwrap();

        let emitter = TestEmitter::new();
        let result = orchestrator
            .search_word_progressive_with_emitter("progressive", &db, &emitter)
            .await;

        assert!(result.is_ok());

        // Verify section 1 data
        let section1 = emitter.get_section1().unwrap();
        assert_eq!(section1.word, "progressive");
        assert_eq!(section1.formality.level, "Formal");
        assert_eq!(section1.domains.len(), 1);
        assert_eq!(section1.domains[0], "Technology");

        // Verify section 2 data
        let section2 = emitter.get_section2().unwrap();
        assert_eq!(section2.meanings.len(), 1);
        assert_eq!(section2.meanings[0].part_of_speech, "adjective");
        assert_eq!(
            section2.meanings[0].memory_tip,
            "Progress happens progressively"
        );

        // Verify section 3 data
        let section3 = emitter.get_section3().unwrap();
        assert_eq!(section3.synonyms.len(), 2);
        assert_eq!(section3.antonyms.len(), 1);
        assert_eq!(section3.collocations.len(), 1);
    }

    fn create_test_db() -> Database {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).expect("Failed to create test database");
        // Keep temp_dir alive by leaking it (acceptable for tests)
        std::mem::forget(temp_dir);
        db
    }

    fn setup_test_cache(db: &Database) {
        let cached = crate::models::WordProgressiveData {
            section1: WordSection1Header {
                word: "cached".to_string(),
                pronunciation: "/ˈkæʃt/".to_string(),
                syllables: "cached".to_string(),
                origin: "cache".to_string(),
                formality: crate::models::FormalityInfo {
                    level: "Neutral".to_string(),
                    percentage: 50,
                },
                domains: vec![],
                tldr: "Test TL;DR".to_string(),
            },
            section2: WordSection2Meanings {
                meanings: vec![crate::models::MeaningItem {
                    number: 1,
                    part_of_speech: "verb".to_string(),
                    brief_gloss: String::new(),
                    definition: "Stored for quick retrieval".to_string(),
                    memory_tip: "Think of a treasure cache".to_string(),
                    examples: vec!["The data is cached".to_string()],
                }],
            },
            mistakes: None,
            section3: WordSection3Related {
                synonyms: vec!["stored".to_string()],
                antonyms: vec![],
                collocations: vec![],
            },
        };
        db.cache_word_progressive("cached", &cached, "openai")
            .unwrap();
    }
}

#[cfg(test)]
mod check_spelling_tests {
    use crate::services::command_orchestrator::CommandOrchestrator;
    use crate::services::provider_registry::ProviderRegistry;

    #[tokio::test]
    async fn test_check_spelling_correct_word() {
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);

        let result = orchestrator.check_spelling("running").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.original_word, "running");
        assert!(response.is_correct);
        assert!(response.suggested_word.is_none());
        assert!(response.alternatives.is_empty());
    }

    #[tokio::test]
    async fn test_check_spelling_misspelled_word() {
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);

        let result = orchestrator.check_spelling("runing").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.original_word, "runing");
        assert!(!response.is_correct);
        assert!(response.suggested_word.is_some());

        // Suggested word should be "running"
        let suggested = response.suggested_word.unwrap();
        assert!(suggested == "running" || suggested == "ruing");

        // Alternatives should include "running"
        assert!(!response.alternatives.is_empty());
        assert!(response.alternatives.iter().any(|w| w == "running"));
    }

    #[tokio::test]
    async fn test_check_spelling_case_insensitive() {
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);

        let result = orchestrator.check_spelling("RUNNING").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.original_word, "RUNNING");
        assert!(response.is_correct);
    }

    #[tokio::test]
    async fn test_check_spelling_edge_cases() {
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);

        // Empty string
        let result = orchestrator.check_spelling("").await;
        assert!(result.is_ok());

        // Single character
        let result = orchestrator.check_spelling("a").await;
        assert!(result.is_ok());

        // Non-English characters
        let result = orchestrator.check_spelling("café").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_spelling_integration_flow() {
        // Test the full UX flow: check spelling, show dialog, user selects
        let registry = ProviderRegistry::new();
        let orchestrator = CommandOrchestrator::new(registry);

        // Step 1: User types "recieve" (common mistake)
        let result = orchestrator.check_spelling("recieve").await;
        assert!(result.is_ok());

        let response = result.unwrap();

        // Step 2: Should detect misspelling
        assert!(!response.is_correct);
        assert!(response.suggested_word.is_some());

        // Step 3: Should provide "receive" as alternative
        let suggested = response.suggested_word.unwrap();
        assert_eq!(suggested, "receive");

        // Step 4: User selects correct word, goes to search_word (not tested here)
        // The selected word would be passed directly to search_word_progressive
    }
}
