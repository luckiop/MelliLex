use crate::models::{
    WordSection1Header, WordSection2Meanings, WordMistakes, WordSection3Related,
    TokenUsage, TranslateResult, ZhEnResult,
};
use crate::services::ai_provider::PromptSender;
use crate::services::prompt_manager::PromptManager;
use crate::services::response_parser::ResponseParser;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct DictionaryService {
    ai_provider: Arc<dyn PromptSender>,
    prompt_manager: PromptManager,
}

impl DictionaryService {
    pub fn new(provider: Arc<dyn PromptSender>) -> Self {
        DictionaryService {
            ai_provider: provider,
            prompt_manager: PromptManager::new(),
        }
    }

    pub async fn generate_section1_header(
        &self,
        word: &str,
        language: &str,
    ) -> Result<(WordSection1Header, Option<TokenUsage>)> {
        log::debug!(
            "Generating section 1 (header) for: {} in {}",
            word,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("section1_header", word, language)
            .ok_or_else(|| anyhow::anyhow!("Template 'section1_header' not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let header: WordSection1Header = ResponseParser::parse_partial(&response, "section1_header")?;

        log::info!(
            "Section 1 completed: formality={}, domains={}",
            header.formality.level,
            header.domains.len()
        );
        log_token_usage(&token_usage, "Section 1 (header)");

        Ok((header, token_usage))
    }

    pub async fn generate_section2_meanings(
        &self,
        word: &str,
        language: &str,
        technical_query: bool,
        technical_domain: &str,
    ) -> Result<(WordSection2Meanings, Option<TokenUsage>)> {
        log::debug!(
            "Generating section 2 (meanings) for: {} in {} (technical={}, domain={})",
            word,
            language,
            technical_query,
            technical_domain
        );

        let prompt = self
            .prompt_manager
            .render_section2_meanings(word, language, technical_query, technical_domain)
            .ok_or_else(|| anyhow::anyhow!("Template 'section2_meanings' not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let meanings: WordSection2Meanings =
            ResponseParser::parse_partial(&response, "section2_meanings")?;

        log::info!(
            "Section 2 completed: {} meanings with definitions",
            meanings.meanings.len()
        );
        log_token_usage(&token_usage, "Section 2 (meanings)");

        Ok((meanings, token_usage))
    }

    pub async fn generate_section3_mistakes(
        &self,
        word: &str,
        language: &str,
    ) -> Result<(WordMistakes, Option<TokenUsage>)> {
        log::debug!(
            "Generating section 3 (mistakes) for: {} in {}",
            word,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("section3_mistakes", word, language)
            .ok_or_else(|| anyhow::anyhow!("Template 'section3_mistakes' not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let mistakes: WordMistakes =
            ResponseParser::parse_partial(&response, "section3_mistakes")?;

        log::info!(
            "Section 3 completed: {} common mistakes",
            mistakes.mistakes.len()
        );
        log_token_usage(&token_usage, "Section 3 (mistakes)");

        Ok((mistakes, token_usage))
    }

    pub async fn generate_section3_related(
        &self,
        word: &str,
        language: &str,
    ) -> Result<(WordSection3Related, Option<TokenUsage>)> {
        log::debug!(
            "Generating section 3 (related words) for: {} in {}",
            word,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("section3_related", word, language)
            .ok_or_else(|| anyhow::anyhow!("Template 'section3_related' not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let mut related: WordSection3Related =
            ResponseParser::parse_partial(&response, "section3_related")?;
        related = Self::english_only_related(related);
        Self::ensure_related_has_entries(word, &related, &response)?;

        log::info!(
            "Section 3 completed: {} synonyms, {} antonyms, {} collocations",
            related.synonyms.len(),
            related.antonyms.len(),
            related.collocations.len()
        );
        log_token_usage(&token_usage, "Section 3 (related)");

        Ok((related, token_usage))
    }

    pub async fn generate_zh_en(&self, query: &str) -> Result<(ZhEnResult, Option<TokenUsage>)> {
        log::info!("Generating zh-en equivalents for: {}", query);

        let prompt = self
            .prompt_manager
            .render_zh_en(query)
            .ok_or_else(|| anyhow::anyhow!("Template 'zh_en_equivalents' not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let mut result: ZhEnResult =
            ResponseParser::parse_partial(&response, "zh_en_equivalents")?;
        if result.query.trim().is_empty() {
            result.query = query.to_string();
        }
        result.equivalents.truncate(5);
        if result.equivalents.is_empty() {
            return Err(anyhow::anyhow!("Zh-en payload had no equivalents"));
        }

        log::info!(
            "Zh-en completed: {} equivalents for '{}'",
            result.equivalents.len(),
            result.query
        );
        log_token_usage(&token_usage, "Zh-en equivalents");

        Ok((result, token_usage))
    }

    pub async fn generate_translate(
        &self,
        source: &str,
    ) -> Result<(TranslateResult, Option<TokenUsage>)> {
        log::info!("Generating translation ({} chars)", source.chars().count());

        let prompt = self
            .prompt_manager
            .render_translate(source)
            .ok_or_else(|| anyhow::anyhow!("Template 'translate' not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let mut result: TranslateResult =
            ResponseParser::parse_partial(&response, "translate")?;
        if result.source.trim().is_empty() {
            result.source = source.to_string();
        }
        result.translation = result.translation.trim().to_string();
        if result.translation.is_empty() {
            return Err(anyhow::anyhow!("Translate payload had no translation"));
        }
        if let Some(remark) = result.remark.as_mut() {
            *remark = remark.trim().to_string();
            if remark.is_empty() {
                result.remark = None;
            }
        }

        log::info!(
            "Translate completed: {} chars Chinese, remark={}",
            result.translation.chars().count(),
            result.remark.is_some()
        );
        log_token_usage(&token_usage, "Translate");

        Ok((result, token_usage))
    }

    fn english_only_related(related: WordSection3Related) -> WordSection3Related {
        WordSection3Related {
            synonyms: related
                .synonyms
                .into_iter()
                .filter(|item| is_english_lexeme(item))
                .collect(),
            antonyms: related
                .antonyms
                .into_iter()
                .filter(|item| is_english_lexeme(item))
                .collect(),
            collocations: related
                .collocations
                .into_iter()
                .filter(|item| is_english_lexeme(&item.phrase))
                .collect(),
        }
    }

    fn ensure_related_has_entries(
        word: &str,
        related: &WordSection3Related,
        response: &str,
    ) -> Result<()> {
        let has_entries = !related.synonyms.is_empty()
            || !related.antonyms.is_empty()
            || !related.collocations.is_empty();

        if has_entries {
            return Ok(());
        }

        let preview: String = response.chars().take(400).collect();
        log::warn!(
            "Section 3 payload was empty for '{}'. Response preview: {}",
            word,
            preview
        );
        Err(anyhow::anyhow!(format!(
            "Section 3 payload empty for '{}'",
            word
        )))
    }
}

fn contains_han(value: &str) -> bool {
    value.chars().any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch))
}

fn is_english_lexeme(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || contains_han(trimmed) {
        return false;
    }
    let mut seen_letter = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphabetic() {
            seen_letter = true;
            continue;
        }
        if matches!(ch, ' ' | '-' | '\'') {
            continue;
        }
        return false;
    }
    seen_letter
}

fn log_token_usage(usage: &Option<TokenUsage>, context: &str) {
    if let Some(tokens) = usage {
        log::info!(
            "{} - Total tokens: {} (prompt: {}, completion: {})",
            context,
            tokens.total_tokens,
            tokens.prompt_tokens,
            tokens.completion_tokens
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_related_requires_entries() {
        let empty = WordSection3Related {
            synonyms: vec![],
            antonyms: vec![],
            collocations: vec![],
        };

        let err = DictionaryService::ensure_related_has_entries("candor", &empty, "{}")
            .expect_err("empty payload should be rejected");
        assert!(err
            .to_string()
            .contains("Section 3 payload empty for 'candor'"));
    }

    #[test]
    fn ensure_related_accepts_valid_payload() {
        let section = WordSection3Related {
            synonyms: vec!["sincerity".into()],
            antonyms: vec![],
            collocations: vec![],
        };

        assert!(
            DictionaryService::ensure_related_has_entries("candor", &section, "{}").is_ok(),
            "non-empty payload should be accepted"
        );
    }

    #[test]
    fn drops_han_related_terms() {
        let section = WordSection3Related {
            synonyms: vec!["mechanical".into(), "机械的".into()],
            antonyms: vec!["organic".into()],
            collocations: vec![
                crate::models::CollocationItem {
                    phrase: "机械臂".into(),
                    example: "x".into(),
                },
                crate::models::CollocationItem {
                    phrase: "mechanical arm".into(),
                    example: "x".into(),
                },
            ],
        };
        let filtered = DictionaryService::english_only_related(section);
        assert_eq!(filtered.synonyms, vec!["mechanical".to_string()]);
        assert_eq!(filtered.collocations.len(), 1);
        assert_eq!(filtered.collocations[0].phrase, "mechanical arm");
        assert!(is_english_lexeme("self-control"));
        assert!(!is_english_lexeme("メカニカル"));
    }
}
