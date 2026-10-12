use crate::errors::AppError;
use crate::models::SpellCheckResponse;
use crate::services::text_processing::{
    spell_checker::SymSpellChecker,
    word_forms::StemmerAnalyzer,
    TextProcessor,
};
use crate::validation;
use std::sync::Arc;

/// Single Responsibility: Coordinate spell checking and word variation operations
pub struct SpellCheckCoordinator {
    text_processor: Arc<TextProcessor>,
}

impl SpellCheckCoordinator {
    pub fn new() -> Self {
        let spell_checker = SymSpellChecker::new().expect("Failed to load English spell checker");
        let stemmer = StemmerAnalyzer::new("english").expect("Failed to load stemmer");
        let text_processor = Arc::new(TextProcessor::new(
            Arc::new(spell_checker),
            Arc::new(stemmer),
        ));

        Self { text_processor }
    }

    pub async fn check_spelling(&self, word: &str) -> Result<SpellCheckResponse, AppError> {
        // Handle edge cases gracefully for spell checking
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return Ok(SpellCheckResponse {
                original_word: word.to_string(),
                is_correct: true,
                suggested_word: None,
                alternatives: vec![],
            });
        }

        // For non-empty words, validate normally
        let validated_word = validation::validate_word_query(trimmed)?;
        log::info!("Checking spelling for word: {}", validated_word);

        let result = self
            .text_processor
            .preprocess_word(&validated_word)
            .map_err(|e| AppError::validation(format!("Spell check failed: {}", e)))?;

        Ok(SpellCheckResponse {
            original_word: result.original,
            is_correct: !result.is_misspelled,
            suggested_word: result.corrected,
            alternatives: result.suggestions,
        })
    }

    pub fn suggest_words(&self, query: &str) -> Vec<String> {
        self.text_processor.suggest_words(query)
    }

    pub async fn get_word_variations(&self, word: &str) -> Result<Vec<String>, AppError> {
        let validated_word = validation::validate_word_query(word)?;
        self.text_processor
            .get_highlight_patterns(&validated_word)
            .map_err(|e| {
                log::error!(
                    "Failed to get word variations for '{}': {}",
                    validated_word,
                    e
                );
                AppError::from(e)
            })
    }
}

impl Default for SpellCheckCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
