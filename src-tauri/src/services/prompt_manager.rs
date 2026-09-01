use std::collections::HashMap;

use crate::models::SUPPORTED_LANGUAGES;

/// Extra instruction appended to the meanings prompt when technical query is on.
pub const TECHNICAL_MEANING_INSTRUCTION: &str = "\
TECHNICAL SENSE (required, first item):\n\
Also give a specialist definition aimed at computer science, control engineering, and robotics \
(software, algorithms, signals, kinematics, actuators, ROS, etc.).\n\
- Put that technical sense FIRST in the \"meanings\" array as number 1.\n\
- Then list up to 3 general-language senses in frequency order (up to 4 meanings total).\n\
- The technical definition must name the field, be one precise sentence, and use domain examples.\n\
- If the word is not a standard term in those fields, meaning 1 should say so briefly and give the \
closest technical usage, then continue with general senses.\n\
- Keep the same JSON schema; do not drop briefGloss, memoryTip, or examples.\n\
- Technical briefGloss should be a short domain label (e.g. 增益), not a full sentence.";

#[derive(Clone)]
pub struct PromptManager {
    templates: HashMap<String, String>,
}

impl PromptManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Exploration prompts
        templates.insert(
            "formality_analysis".to_string(),
            include_str!("../../prompts/formality_analysis.txt").to_string(),
        );
        templates.insert(
            "usage_patterns".to_string(),
            include_str!("../../prompts/usage_patterns.txt").to_string(),
        );
        templates.insert(
            "domain_exploration".to_string(),
            include_str!("../../prompts/domain_exploration.txt").to_string(),
        );
        templates.insert(
            "practice_exercises".to_string(),
            include_str!("../../prompts/practice_exercises.txt").to_string(),
        );
        templates.insert(
            "contextual_examples".to_string(),
            include_str!("../../prompts/contextual_examples.txt").to_string(),
        );

        // New 4-section progressive prompts
        templates.insert(
            "section1_header".to_string(),
            include_str!("../../prompts/section1_header.txt").to_string(),
        );
        templates.insert(
            "section2_meanings".to_string(),
            include_str!("../../prompts/section2_meanings.txt").to_string(),
        );
        templates.insert(
            "section3_mistakes".to_string(),
            include_str!("../../prompts/section3_mistakes.txt").to_string(),
        );
        templates.insert(
            "section3_related".to_string(),
            include_str!("../../prompts/section3_related.txt").to_string(),
        );

        // Phrase lookup prompts
        templates.insert(
            "phrase_section1_overview".to_string(),
            include_str!("../../prompts/phrase_section1_overview.txt").to_string(),
        );
        templates.insert(
            "phrase_section2_context".to_string(),
            include_str!("../../prompts/phrase_section2_context.txt").to_string(),
        );
        templates.insert(
            "phrase_section3_related".to_string(),
            include_str!("../../prompts/phrase_section3_related.txt").to_string(),
        );
        templates.insert(
            "zh_en_equivalents".to_string(),
            include_str!("../../prompts/zh_en_equivalents.txt").to_string(),
        );

        PromptManager { templates }
    }

    fn render(&self, template_name: &str, word: &str) -> Option<String> {
        self.templates
            .get(template_name)
            .map(|template| replace_placeholders(template, word, None))
    }

    pub fn render_zh_en(&self, query: &str) -> Option<String> {
        self.render("zh_en_equivalents", query)
    }

    pub fn render_with_language(
        &self,
        template_name: &str,
        word: &str,
        language: &str,
    ) -> Option<String> {
        let mut prompt = self.render(template_name, word)?;

        if should_apply_language_instruction(language) {
            let language_instruction = self.get_language_instruction(language);
            prompt = format!("{}\n\n{}", language_instruction, prompt);
        } else if !language.eq_ignore_ascii_case("English") {
            log::warn!(
                "[PromptManager] Unsupported language '{}', falling back to English prompts",
                language
            );
        }

        Some(prompt)
    }

    /// Meanings prompt, with an optional CS/control/robotics sense pinned first.
    pub fn render_section2_meanings(
        &self,
        word: &str,
        language: &str,
        technical_query: bool,
    ) -> Option<String> {
        let mut prompt = self.render_with_language("section2_meanings", word, language)?;
        if technical_query {
            prompt = format!("{}\n\n{}", prompt, TECHNICAL_MEANING_INSTRUCTION);
        }
        Some(prompt)
    }

    fn get_language_instruction(&self, language: &str) -> String {
        format!(
                "LANGUAGE INSTRUCTION:\n\
                 1. Write EVERY descriptive string value (definitions, briefGloss labels, explanations, contexts, descriptions, questions, answers, memory tips, collocations) entirely in {}.\n\
                 2. Keep JSON keys and enumerated values exactly as specified (e.g., level, patternType, exerciseType stay in English).\n\
                 3. Keep the English headword itself in English.\n\
                 4. Example sentences should remain in English, followed immediately by a {} translation in parentheses.\n\
                 5. Never fall back to English for descriptive text—only the schematic labels stay English.\n\
                 6. Use natural, learner-friendly {} and reference cultural touchpoints familiar to {} speakers when helpful.\n\
                 7. CRITICAL: Maintain the exact JSON schema from the prompt. If the schema requires a string array (e.g., [\"word1\", \"word2\"]), do NOT return objects (e.g., [{{\"word\": \"...\", \"definition\": \"...\"}}]). Arrays must contain only simple strings unless the schema explicitly shows otherwise.\n\
                 8. Ensure valid UTF-8 characters and proper JSON escaping.\n\
                 \n\
                 Example format (for guidance only):\n\
                 - briefGloss: \"[2–8 character {} label, not a sentence]\"\n\
                 - definition: \"[Definition written in {}]\"\n\
                 - memoryTip: \"[Mnemonic in {} referencing local culture]\"\n\
                 - example: \"She jumped for joy. ([{} translation here])\"\n\
                 - relatedWords: [\"word1\", \"word2\", \"word3\"] (NOT objects with definitions)",
                language, language, language, language, language, language, language, language
            )
    }

    fn render_with_context(
        &self,
        template_name: &str,
        word: &str,
        context: &str,
    ) -> Option<String> {
        self.templates
            .get(template_name)
            .map(|template| replace_placeholders(template, word, Some(context)))
    }

    pub fn render_with_context_and_language(
        &self,
        template_name: &str,
        word: &str,
        context: &str,
        language: &str,
    ) -> Option<String> {
        let mut prompt = self.render_with_context(template_name, word, context)?;

        if should_apply_language_instruction(language) {
            let language_instruction = self.get_language_instruction(language);
            prompt = format!("{}\n\n{}", language_instruction, prompt);
        } else if !language.eq_ignore_ascii_case("English") {
            log::warn!(
                "[PromptManager] Unsupported language '{}', falling back to English prompts",
                language
            );
        }

        Some(prompt)
    }

}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

fn should_apply_language_instruction(language: &str) -> bool {
    if language.eq_ignore_ascii_case("English") {
        return false;
    }

    let normalized = language.trim();
    SUPPORTED_LANGUAGES.iter().any(|(name, code)| {
        name.eq_ignore_ascii_case(normalized) || code.eq_ignore_ascii_case(normalized)
    })
}

fn replace_placeholders(template: &str, word: &str, context: Option<&str>) -> String {
    let mut rendered = template
        .replace("{{word}}", word)
        .replace("{{phrase}}", word);

    if let Some(context_value) = context {
        rendered = rendered.replace("{{context}}", context_value);
    }

    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_all_templates() {
        let manager = PromptManager::new();
        assert_eq!(
            manager.templates.len(),
            13,
            "Expected 13 templates (5 exploration + 4 word sections + 3 phrase sections + zh-en)"
        );
        // Exploration templates
        assert!(manager.templates.contains_key("formality_analysis"));
        assert!(manager.templates.contains_key("usage_patterns"));
        assert!(manager.templates.contains_key("domain_exploration"));
        assert!(manager.templates.contains_key("practice_exercises"));
        assert!(manager.templates.contains_key("contextual_examples"));
        // Word section templates
        assert!(manager.templates.contains_key("section1_header"));
        assert!(manager.templates.contains_key("section2_meanings"));
        assert!(manager.templates.contains_key("section3_mistakes"));
        assert!(manager.templates.contains_key("section3_related"));
        // Phrase section templates
        assert!(manager.templates.contains_key("phrase_section1_overview"));
        assert!(manager.templates.contains_key("phrase_section2_context"));
        assert!(manager.templates.contains_key("phrase_section3_related"));
        assert!(manager.templates.contains_key("zh_en_equivalents"));
    }

    #[test]
    fn render_template_with_word() {
        let manager = PromptManager::new();
        let prompt = manager
            .render("section1_header", "elaborate")
            .expect("section1_header template not found");

        assert!(prompt.contains("elaborate"));
        assert!(prompt.contains("expert lexicographer"));
        assert!(!prompt.contains("{{word}}"));
    }

    #[test]
    fn all_prompt_methods_work() {
        let manager = PromptManager::new();

        assert!(manager
            .render_with_language("section1_header", "test", "English")
            .expect("section1_header template not found")
            .contains("test"));
        let section2 = manager
            .render_with_language("section2_meanings", "test", "English")
            .expect("section2_meanings template not found");
        assert!(section2.contains("test"));
        assert!(section2.contains("briefGloss"));
        assert!(manager
            .render_with_language("phrase_section1_overview", "test phrase", "English")
            .expect("phrase_section1_overview template not found")
            .contains("test phrase"));
    }

    #[test]
    fn section2_technical_query_pins_specialist_sense() {
        let manager = PromptManager::new();
        let normal = manager
            .render_section2_meanings("gain", "English", false)
            .expect("section2");
        let technical = manager
            .render_section2_meanings("gain", "English", true)
            .expect("section2 technical");

        assert!(normal.contains("gain"));
        assert!(!normal.contains("TECHNICAL SENSE"));
        assert!(technical.contains("TECHNICAL SENSE"));
        assert!(technical.contains("robotics"));
        assert!(technical.contains("briefGloss"));
        assert!(technical.starts_with(&normal));
    }

    #[test]
    fn section2_prompt_requires_brief_gloss() {
        let manager = PromptManager::new();
        let prompt = manager
            .render_section2_meanings("microcontroller", "Chinese (Simplified)", false)
            .expect("section2");
        assert!(prompt.contains("briefGloss"));
        assert!(prompt.contains("简要含义"));
        assert!(prompt.contains("微控制器"));
        assert!(prompt.contains("不要写成完整句子"));
    }

    #[test]
    fn render_zh_en_inserts_query() {
        let manager = PromptManager::new();
        let prompt = manager.render_zh_en("餐桌").expect("zh_en template");
        assert!(prompt.contains("餐桌"));
        assert!(prompt.contains("equivalents"));
        assert!(!prompt.contains("{{word}}"));
    }
}
