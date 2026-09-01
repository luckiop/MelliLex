use crate::models::{
    PhraseDefinitionData, PhraseSection1Overview, PhraseSection2Context, PhraseSection3Related,
    WordProgressiveData, WordSection1Header, WordSection2Meanings, WordSection3Related,
};

/// Single Responsibility: Format WordProgressiveData into markdown text
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn format(word: &str, data: &WordProgressiveData, include_timestamp: bool) -> String {
        let mut md = String::new();

        // Title
        md.push_str(&format!("# {}\n\n", word));

        // Section 1: Header
        md.push_str(&Self::format_header_section(&data.section1));
        md.push_str("\n\n");

        // Section 2: Meanings
        md.push_str(&Self::format_meanings_section(&data.section2));
        md.push_str("\n\n");

        // Section 3: Related Words
        md.push_str(&Self::format_related_section(&data.section3));

        // Optional timestamp
        if include_timestamp {
            md.push_str("\n\n---\n");
            md.push_str(&format!(
                "*Exported: {}*\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }

        md
    }

    fn format_header_section(header: &WordSection1Header) -> String {
        let mut md = String::new();

        // Pronunciation and basic info
        if !header.pronunciation.is_empty() {
            md.push_str(&format!("**Pronunciation:** {}\n\n", header.pronunciation));
        }

        if !header.syllables.is_empty() {
            md.push_str(&format!("**Syllables:** {}\n\n", header.syllables));
        }

        // TL;DR
        if !header.tldr.is_empty() {
            md.push_str("## In Simple Words\n\n");
            md.push_str(&format!("{}\n\n", header.tldr));
        }

        // Formality
        md.push_str(&format!(
            "**Formality:** {} ({}%)\n\n",
            header.formality.level, header.formality.percentage
        ));

        // Domains
        if !header.domains.is_empty() {
            md.push_str(&format!("**Used in:** {}\n\n", header.domains.join(", ")));
        }

        // Origin
        if !header.origin.is_empty() {
            md.push_str(&format!("**Origin:** {}\n", header.origin));
        }

        md
    }

    fn format_meanings_section(meanings: &WordSection2Meanings) -> String {
        let mut md = String::new();
        md.push_str("## Meanings\n\n");

        for meaning in &meanings.meanings {
            md.push_str(&format!(
                "### {}. {} ({})\n\n",
                meaning.number, meaning.definition, meaning.part_of_speech
            ));

            // Memory tip
            if !meaning.memory_tip.is_empty() {
                md.push_str(&format!("**Memory Tip:** {}\n\n", meaning.memory_tip));
            }

            // Examples
            if !meaning.examples.is_empty() {
                md.push_str("**Examples:**\n\n");
                for example in &meaning.examples {
                    md.push_str(&format!("- {}\n", example));
                }
                md.push('\n');
            }
        }

        md
    }

    fn format_related_section(related: &WordSection3Related) -> String {
        let mut md = String::new();
        md.push_str("## Related Words\n\n");

        // Synonyms
        if !related.synonyms.is_empty() {
            md.push_str("**Synonyms:** ");
            md.push_str(&related.synonyms.join(", "));
            md.push_str("\n\n");
        }

        // Antonyms
        if !related.antonyms.is_empty() {
            md.push_str("**Antonyms:** ");
            md.push_str(&related.antonyms.join(", "));
            md.push_str("\n\n");
        }

        // Collocations
        if !related.collocations.is_empty() {
            md.push_str("**Common Collocations:**\n\n");
            for collocation in &related.collocations {
                md.push_str(&format!(
                    "- **{}**: {}\n",
                    collocation.phrase, collocation.example
                ));
            }
        }

        md
    }

    // ============== PHRASE FORMATTING ==============

    pub fn format_phrase(
        phrase: &str,
        data: &PhraseDefinitionData,
        include_timestamp: bool,
    ) -> String {
        let mut md = String::new();

        // Title
        md.push_str(&format!("# {}\n\n", phrase));

        // Section 1: Overview
        md.push_str(&Self::format_phrase_overview(&data.section1));
        md.push('\n');

        // Section 2: Context
        md.push_str(&Self::format_phrase_context(&data.section2));
        md.push('\n');

        // Section 3: Related
        md.push_str(&Self::format_phrase_related(&data.section3));

        // Optional timestamp
        if include_timestamp {
            md.push_str("\n---\n");
            md.push_str(&format!(
                "*Exported: {}*\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }

        md
    }

    fn format_phrase_overview(overview: &PhraseSection1Overview) -> String {
        let mut md = String::new();

        // Metadata line
        md.push_str(&format!(
            "**Type:** {} · **Region:** {} · **Formality:** {}\n\n",
            overview.phrase_type, overview.region, overview.formality.level
        ));

        // TL;DR
        md.push_str("## In a Nutshell\n\n");
        md.push_str(&format!("{}\n\n", overview.tldr));

        // Literal meaning (if exists)
        if let Some(ref literal) = overview.literal_meaning {
            md.push_str("## Literal Meaning\n\n");
            md.push_str(&format!("{}\n\n", literal));
        }

        // Actual meaning
        md.push_str("## Actual Meaning\n\n");
        md.push_str(&format!("{}\n", overview.actual_meaning));

        md
    }

    fn format_phrase_context(context: &PhraseSection2Context) -> String {
        let mut md = String::new();

        // Origin Story
        md.push_str("## Origin Story\n\n");
        if let Some(ref era) = context.origin.era {
            md.push_str(&format!("**Era:** {}\n\n", era));
        }
        if let Some(ref source) = context.origin.source {
            md.push_str(&format!("**Source:** {}\n\n", source));
        }
        md.push_str(&format!("{}\n", context.origin.story));
        if let Some(ref evolution) = context.origin.evolution {
            md.push_str(&format!("\n**Evolution:** {}\n", evolution));
        }
        md.push('\n');

        // Usage Notes
        if !context.usage_notes.is_empty() {
            md.push_str("## Usage Notes\n\n");
            for note in &context.usage_notes {
                let tone_str = note
                    .tone
                    .as_ref()
                    .map(|t| format!(" ({})", t))
                    .unwrap_or_default();
                md.push_str(&format!("### {}{}\n\n", note.context, tone_str));
                md.push_str(&format!("> {}\n\n", note.example));
            }
        }

        // Common Mistakes
        if !context.common_mistakes.is_empty() {
            md.push_str("## Common Mistakes\n\n");
            for mistake in &context.common_mistakes {
                md.push_str(&format!("### {}\n\n", mistake.mistake_type));
                md.push_str(&format!("Incorrect: {}\n\n", mistake.incorrect));
                md.push_str(&format!("Correct: {}\n\n", mistake.correct));
                md.push_str(&format!("> {}\n\n", mistake.explanation));
            }
        }

        md
    }

    fn format_phrase_related(related: &PhraseSection3Related) -> String {
        let mut md = String::new();

        // Variations
        if !related.variations.is_empty() {
            md.push_str("## Variations\n\n");
            for variation in &related.variations {
                let region_str = variation
                    .region
                    .as_ref()
                    .map(|r| format!(" ({})", r))
                    .unwrap_or_default();
                let note_str = variation
                    .note
                    .as_ref()
                    .map(|n| format!(" — {}", n))
                    .unwrap_or_default();
                md.push_str(&format!(
                    "- **{}**{}{}\n",
                    variation.phrase, note_str, region_str
                ));
            }
            md.push('\n');
        }

        // Similar Phrases
        if !related.similar_phrases.is_empty() {
            md.push_str("## Similar Phrases\n\n");
            for phrase in &related.similar_phrases {
                md.push_str(&format!("- **{}** — {}\n", phrase.phrase, phrase.meaning_hint));
            }
            md.push('\n');
        }

        // Opposite Phrases
        if !related.opposite_phrases.is_empty() {
            md.push_str("## Opposite Phrases\n\n");
            for phrase in &related.opposite_phrases {
                md.push_str(&format!("- **{}** — {}\n", phrase.phrase, phrase.meaning_hint));
            }
            md.push('\n');
        }

        // See Also
        if !related.see_also.is_empty() {
            md.push_str("## See Also\n\n");
            for item in &related.see_also {
                md.push_str(&format!("- {}\n", item));
            }
            md.push('\n');
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn create_test_data() -> WordProgressiveData {
        WordProgressiveData {
            section1: WordSection1Header {
                word: "ephemeral".to_string(),
                pronunciation: "/ɪˈfɛm.ə.rəl/".to_string(),
                syllables: "e·phem·er·al".to_string(),
                origin: "Greek ephēmeros 'lasting only a day'".to_string(),
                formality: FormalityInfo {
                    level: "Formal".to_string(),
                    percentage: 75,
                },
                domains: vec!["Literature".to_string(), "Philosophy".to_string()],
                tldr: "Something that lasts for a very short time".to_string(),
            },
            section2: WordSection2Meanings {
                meanings: vec![MeaningItem {
                    number: 1,
                    part_of_speech: "adjective".to_string(),
                    brief_gloss: "短暂的".to_string(),
                    definition: "Lasting for a very short time".to_string(),
                    memory_tip: "Think of 'ephemeral' as something that disappears quickly, like morning dew".to_string(),
                    examples: vec![
                        "The beauty of cherry blossoms is ephemeral.".to_string(),
                    ],
                }],
            },
            mistakes: None,
            section3: WordSection3Related {
                synonyms: vec!["transient".to_string(), "fleeting".to_string()],
                antonyms: vec!["permanent".to_string(), "enduring".to_string()],
                collocations: vec![CollocationItem {
                    phrase: "ephemeral nature".to_string(),
                    example: "The ephemeral nature of fame".to_string(),
                }],
            },
        }
    }

    #[test]
    fn test_format_markdown() {
        let data = create_test_data();
        let md = MarkdownFormatter::format("ephemeral", &data, false);

        assert!(md.contains("# ephemeral"));
        assert!(md.contains("**Pronunciation:**"));
        assert!(md.contains("## Meanings"));
        assert!(md.contains("## Related Words"));
    }
}
