use super::{SpellChecker, WordFormsAnalyzer};
use anyhow::Result;
use std::sync::Arc;

pub struct TextProcessor {
    spell_checker: Arc<dyn SpellChecker>,
    word_forms: Arc<dyn WordFormsAnalyzer>,
}

impl TextProcessor {
    pub fn new(
        spell_checker: Arc<dyn SpellChecker>,
        word_forms: Arc<dyn WordFormsAnalyzer>,
    ) -> Self {
        Self {
            spell_checker,
            word_forms,
        }
    }

    pub fn suggest_words(&self, query: &str) -> Vec<String> {
        self.spell_checker.suggest(query, 8)
    }

    pub fn preprocess_word(&self, word: &str) -> Result<PreprocessResult> {
        let spell_check = self.spell_checker.check(word)?;

        if !spell_check.is_correct && !spell_check.suggestions.is_empty() {
            let suggestion = &spell_check.suggestions[0];
            return Ok(PreprocessResult {
                original: word.to_string(),
                corrected: Some(suggestion.clone()),
                is_misspelled: true,
                suggestions: spell_check.suggestions.clone(),
            });
        }

        Ok(PreprocessResult {
            original: word.to_string(),
            corrected: None,
            is_misspelled: false,
            suggestions: vec![],
        })
    }

    pub fn get_highlight_patterns(&self, word: &str) -> Result<Vec<String>> {
        self.word_forms.get_variations(word)
    }
}

#[derive(Debug, Clone)]
pub struct PreprocessResult {
    pub original: String,
    pub corrected: Option<String>,
    pub is_misspelled: bool,
    pub suggestions: Vec<String>,
}

impl PreprocessResult {
    pub fn word_for_llm(&self) -> &str {
        self.corrected.as_deref().unwrap_or(&self.original)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::text_processing::spell_checker::SymSpellChecker;
    use crate::services::text_processing::word_forms::StemmerAnalyzer;

    #[test]
    fn test_text_processor_creation() {
        let spell_checker_result = SymSpellChecker::new();
        let word_forms_result = StemmerAnalyzer::new("english");

        if spell_checker_result.is_err() || word_forms_result.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let spell_checker = Arc::new(spell_checker_result.unwrap());
        let word_forms = Arc::new(word_forms_result.unwrap());

        let processor = TextProcessor::new(spell_checker, word_forms);
        assert_eq!(processor.spell_checker.name(), "SymSpell");
        assert_eq!(processor.word_forms.name(), "Stemmer (Porter/Snowball)");
    }

    #[test]
    fn test_preprocess_word() {
        let spell_checker_result = SymSpellChecker::new();
        let word_forms_result = StemmerAnalyzer::new("english");

        if spell_checker_result.is_err() || word_forms_result.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let spell_checker = Arc::new(spell_checker_result.unwrap());
        let word_forms = Arc::new(word_forms_result.unwrap());
        let processor = TextProcessor::new(spell_checker, word_forms);

        let result = processor.preprocess_word("running").unwrap();
        assert_eq!(result.original, "running");
        assert!(!result.is_misspelled);
    }

    #[test]
    fn test_highlight_patterns() {
        let spell_checker_result = SymSpellChecker::new();
        let word_forms_result = StemmerAnalyzer::new("english");

        if spell_checker_result.is_err() || word_forms_result.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let spell_checker = Arc::new(spell_checker_result.unwrap());
        let word_forms = Arc::new(word_forms_result.unwrap());
        let processor = TextProcessor::new(spell_checker, word_forms);

        let patterns = processor.get_highlight_patterns("run").unwrap();
        assert!(patterns.contains(&"run".to_string()));
        assert!(!patterns.is_empty());
    }


}
