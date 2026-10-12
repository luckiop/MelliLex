use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SpellCheckResult {
    pub is_correct: bool,
    pub original: String,
    pub suggestions: Vec<String>,
}

pub trait SpellChecker: Send + Sync {
    fn check(&self, word: &str) -> Result<SpellCheckResult>;

    fn name(&self) -> &str;

    /// Local prefix and typo suggestions. Empty when the query is not a single English word.
    fn suggest(&self, _query: &str, _limit: usize) -> Vec<String> {
        Vec::new()
    }
}

mod suggest;

pub mod symspell_impl;

pub use symspell_impl::SymSpellChecker;
