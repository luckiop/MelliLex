use super::suggest::{self, DictEntry, FuzzyHit};
use crate::services::text_processing::{SpellCheckResult, SpellChecker};
use crate::utils::resource_path;
use anyhow::Result;
use std::fs;
use std::path::Path;
use symspell::{AsciiStringStrategy, SymSpell, Verbosity};

pub struct SymSpellChecker {
    symspell: SymSpell<AsciiStringStrategy>,
    entries: Vec<DictEntry>,
    max_edit_distance: i64,
}

impl SymSpellChecker {
    pub fn new() -> Result<Self> {
        let dict_path = resource_path::resolve("dictionaries/symspell_frequency_en.txt");
        let dict_path_str = dict_path.to_string_lossy();

        let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();

        if Path::new(dict_path_str.as_ref()).exists() {
            symspell.load_dictionary(&dict_path_str, 0, 1, " ");
        } else {
            return Err(anyhow::anyhow!(
                "English dictionary not found: {}",
                dict_path_str
            ));
        }

        let entries = fs::read_to_string(dict_path_str.as_ref())
            .map(|text| suggest::load_entries(&text))
            .unwrap_or_default();

        Ok(Self {
            symspell,
            entries,
            max_edit_distance: 2,
        })
    }

    pub fn suggest(&self, query: &str, limit: usize) -> Vec<String> {
        let Some(normalized) = suggest::normalize_query(query) else {
            return Vec::new();
        };
        let fuzzy = self
            .symspell
            .lookup(&normalized, Verbosity::All, self.max_edit_distance)
            .into_iter()
            .map(|item| FuzzyHit {
                term: item.term,
                distance: i64::from(item.distance),
                count: item.count,
            })
            .collect::<Vec<_>>();
        suggest::suggest_from(&self.entries, &fuzzy, &normalized, limit)
    }
}

impl SpellChecker for SymSpellChecker {
    fn check(&self, word: &str) -> Result<SpellCheckResult> {
        // Convert to lowercase for case-insensitive checking
        let word_lower = word.to_lowercase();

        // Use Verbosity::Top to get the best suggestion
        let suggestions = self
            .symspell
            .lookup(&word_lower, Verbosity::Top, self.max_edit_distance);

        let is_correct = if suggestions.is_empty() {
            false
        } else {
            // If the top suggestion exactly matches the input, it's correct
            suggestions[0].term.eq_ignore_ascii_case(word)
        };

        let suggestion_terms: Vec<String> = suggestions
            .iter()
            .filter(|s| !s.term.eq_ignore_ascii_case(word))
            .take(5)
            .map(|s| s.term.clone())
            .collect();

        Ok(SpellCheckResult {
            is_correct,
            original: word.to_string(),
            suggestions: suggestion_terms,
        })
    }

    fn name(&self) -> &str {
        "SymSpell"
    }

    fn suggest(&self, query: &str, limit: usize) -> Vec<String> {
        Self::suggest(self, query, limit)
    }
}

impl Default for SymSpellChecker {
    fn default() -> Self {
        Self::new().expect("Failed to create default SymSpellChecker")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symspell_checker_creation() {
        let checker = SymSpellChecker::new();
        assert!(
            checker.is_ok(),
            "Failed to create SymSpellChecker: {:?}",
            checker.err()
        );
    }

    #[test]
    fn test_symspell_basic_check() {
        let checker = SymSpellChecker::new();
        if checker.is_err() {
            eprintln!("Skipping test: dictionary not available");
            return;
        }
        let checker = checker.unwrap();
        let result = checker.check("the").unwrap();
        assert_eq!(result.original, "the");
        assert!(result.is_correct, "Word 'the' should be correct");
    }

    #[test]
    fn test_symspell_misspelling() {
        let checker = SymSpellChecker::new();
        if checker.is_err() {
            eprintln!("Skipping test: dictionary not available");
            return;
        }
        let checker = checker.unwrap();
        let result = checker.check("lamantation").unwrap();
        assert!(
            !result.suggestions.is_empty(),
            "Should have suggestions for misspelling"
        );
    }

    #[test]
    fn suggests_captain_for_prefix_and_transposition() {
        let checker = match SymSpellChecker::new() {
            Ok(checker) => checker,
            Err(_) => {
                eprintln!("Skipping test: dictionary not available");
                return;
            }
        };
        let prefix = checker.suggest("capt", 8);
        assert!(
            prefix.iter().any(|word| word == "captain"),
            "prefix suggestions: {prefix:?}"
        );
        let typo = checker.suggest("captian", 8);
        assert_eq!(
            typo.first().map(String::as_str),
            Some("captain"),
            "typo suggestions: {typo:?}"
        );
        assert!(checker.suggest("c", 8).is_empty());
        assert!(checker.suggest("break the ice", 8).is_empty());
    }
}
