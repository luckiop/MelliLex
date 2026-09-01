use serde::{Deserialize, Serialize};

// Supported languages for explanations
pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("English", "en"),
    ("Spanish", "es"),
    ("Portuguese", "pt"),
    ("French", "fr"),
    ("German", "de"),
    ("Hindi", "hi"),
    ("Arabic", "ar"),
    ("Chinese (Simplified)", "zh"),
    ("Japanese", "ja"),
    ("Korean", "ko"),
    ("Italian", "it"),
    ("Turkish", "tr"),
    ("Russian", "ru"),
];

// Type-safe enums matching frontend TypeScript types

/// Mistake category for common mistakes feature
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MistakeCategory {
    #[default]
    Semantic,
    Grammatical,
    Pronunciation,
    Context,
}

/// Formality level for word alternatives
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum FormalityLevel {
    VeryFormal,
    Formal,
    #[default]
    Neutral,
    Informal,
    VeryInformal,
}

/// Word frequency/commonality level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum WordFrequency {
    VeryCommon,
    #[default]
    Common,
    Uncommon,
    Rare,
    VeryRare,
}

/// Typography mode for UI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TypographyMode {
    Modern,
    #[default]
    Classic,
    Friendly,
}

// Token Usage Tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// Spell Check Model for "Did you mean?" dialog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpellCheckResponse {
    pub original_word: String,
    pub is_correct: bool,
    pub suggested_word: Option<String>,
    pub alternatives: Vec<String>,
}

// Core Data Models - Word progressive structure (3 sections + optional mistakes)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordSection1Header {
    pub word: String,
    pub pronunciation: String,
    pub syllables: String,
    pub origin: String,
    pub formality: FormalityInfo,
    #[serde(default)]
    pub domains: Vec<String>,
    pub tldr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordProgressiveData {
    pub section1: WordSection1Header,
    pub section2: WordSection2Meanings,
    #[serde(default)]
    pub mistakes: Option<WordMistakes>,
    pub section3: WordSection3Related,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormalityInfo {
    pub level: String,
    pub percentage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordSection2Meanings {
    #[serde(default)]
    pub meanings: Vec<MeaningItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeaningItem {
    pub number: u32,
    pub part_of_speech: String,
    pub definition: String,
    pub memory_tip: String,
    #[serde(default)]
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordMistakes {
    #[serde(default)]
    pub mistakes: Vec<MistakeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MistakeItem {
    #[serde(rename = "type")]
    pub mistake_type: String,
    pub incorrect_usage: String,
    pub correction: String,
    #[serde(default)]
    pub category: MistakeCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordSection3Related {
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
    #[serde(default)]
    pub collocations: Vec<CollocationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollocationItem {
    pub phrase: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiWordDefinition {
    pub word: String,
    pub phonetic: Option<String>,
    #[serde(default)]
    pub domain_tags: Vec<String>,
    pub complexity: Option<String>,
    pub frequency: Option<String>,
    pub etymology: Option<Etymology>,
    pub syllable_info: Option<SyllableInfo>,
    pub metrics: Option<Metrics>,
    #[serde(default)]
    pub meanings: Vec<AiMeaning>,
    #[serde(default)]
    pub common_mistakes: Vec<String>,
    pub contextual_usage: Option<ContextualUsage>,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Etymology {
    pub origin: String,
    pub original_form: Option<String>,
    pub meaning: Option<String>,
    #[serde(default)]
    pub evolution_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyllableInfo {
    pub syllable_count: u32,
    #[serde(default)]
    pub syllables: Vec<String>,
    #[serde(default)]
    pub stress_pattern: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub reading_level: Option<f32>,
    pub usage_frequency_rank: Option<u32>,
    pub sentiment_score: Option<f32>,
    pub formality_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiMeaning {
    pub part_of_speech: String,
    pub definitions: Vec<Definition>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
    #[serde(default)]
    pub collocations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    pub text: String,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub contextual_examples: Vec<String>,
    pub memory_tip: Option<String>,
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextualUsage {
    pub formality: Option<String>,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub regional_variations: Vec<RegionalVariation>,
    #[serde(default)]
    pub common_contexts: Vec<String>,
    pub tone_advice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionalVariation {
    pub region: String,
    pub variation: String,
    pub note: Option<String>,
}

// Persistence Models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordHistory {
    pub id: String,
    pub word: String,
    pub timestamp: i64,
    #[serde(rename = "aiProvider")]
    pub ai_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VocabEntry {
    pub word: String,
    pub stars: i32,
    pub category: String,
    pub added_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VocabNoteSnippet {
    #[serde(default)]
    pub part_of_speech: String,
    pub definition: String,
    #[serde(default)]
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub ai_provider: String,
    pub open_ai_config: Option<OpenAiConfig>,
    pub anthropic_config: Option<AnthropicConfig>,
    pub gemini_config: Option<GeminiConfig>,
    pub ollama_config: Option<OllamaConfig>,
    pub theme: String,
    #[serde(default)]
    pub export_settings: Option<ExportSettings>,
    #[serde(default)]
    pub explanation_language: Option<String>,
    #[serde(default)]
    pub ui_language: Option<String>,
    #[serde(default = "default_enable_global_lookup")]
    pub enable_global_lookup: bool,
    #[serde(default = "default_typography_mode")]
    pub typography_mode: String,
    /// When true, the meanings prompt asks for a CS/control/robotics sense first.
    #[serde(default)]
    pub technical_query: bool,
    #[serde(default)]
    pub vocab_note_enabled: bool,
    #[serde(default)]
    pub vocab_note_path: Option<String>,
}

fn default_enable_global_lookup() -> bool {
    true
}

fn default_typography_mode() -> String {
    "classic".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnthropicConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaConfig {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettings {
    #[serde(default)]
    pub include_exploration: bool,
    pub capacities: Option<CapacitiesSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CapacitiesSettings {
    pub api_token: String,
    pub space_id: String,
    #[serde(default)]
    pub default_tags: Vec<String>,
    #[serde(default)]
    pub no_timestamp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormalityAlternative {
    pub word: String,
    #[serde(default)]
    pub level: FormalityLevel,
    pub context: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsagePattern {
    pub template: String,
    pub pattern_type: String,
    pub description: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainExploration {
    pub domain: String,
    #[serde(default)]
    pub usage_frequency: WordFrequency,
    pub common_collocations: Vec<String>,
    pub examples: Vec<String>,
    #[serde(default)]
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeExercise {
    pub question: String,
    pub exercise_type: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub correct_answer: String,
    pub explanation: String,
    #[serde(default)]
    pub is_answered: bool,
    #[serde(default)]
    pub user_answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedFormalityData {
    pub formality_percentage: f64,
    pub formality_alternatives: Vec<FormalityAlternative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedPracticeData {
    pub practice_exercises: Vec<PracticeExercise>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CachedExploreFeatures {
    pub formality: Option<CachedFormalityData>,
    pub domains: Option<Vec<DomainExploration>>,
    pub usage: Option<Vec<UsagePattern>>,
    pub practice: Option<CachedPracticeData>,
}

// ============================================
// Phrase Models - Multi-word expressions
// ============================================

/// Type of phrase/expression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PhraseType {
    #[default]
    Idiom,
    Proverb,
    PhrasalVerb,
    Collocation,
    Expression,
    Saying,
}

impl std::fmt::Display for PhraseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhraseType::Idiom => write!(f, "Idiom"),
            PhraseType::Proverb => write!(f, "Proverb"),
            PhraseType::PhrasalVerb => write!(f, "Phrasal Verb"),
            PhraseType::Collocation => write!(f, "Collocation"),
            PhraseType::Expression => write!(f, "Expression"),
            PhraseType::Saying => write!(f, "Saying"),
        }
    }
}

/// Regional variant of the phrase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PhraseRegion {
    #[default]
    Universal,
    American,
    British,
    Australian,
}

impl std::fmt::Display for PhraseRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhraseRegion::Universal => write!(f, "Universal"),
            PhraseRegion::American => write!(f, "American"),
            PhraseRegion::British => write!(f, "British"),
            PhraseRegion::Australian => write!(f, "Australian"),
        }
    }
}

// Section 1: Overview - "What does it mean?"

/// Quick overview of the phrase meaning and characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseSection1Overview {
    pub phrase: String,
    pub phrase_type: PhraseType,
    pub tldr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_meaning: Option<String>,
    pub actual_meaning: String,
    pub formality: FormalityInfo,
    pub region: PhraseRegion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
}

// Section 2: Context - "Where did it come from?"

/// Historical origin and evolution of the phrase
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseOrigin {
    pub story: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub era: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolution: Option<String>,
}

/// Example usage in a specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseUsageNote {
    pub context: String,
    pub example: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
}

/// Common mistake people make with this phrase
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseMistake {
    pub mistake_type: String,
    pub incorrect: String,
    pub correct: String,
    pub explanation: String,
}

/// Context section with origin story, usage notes, and common mistakes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseSection2Context {
    pub origin: PhraseOrigin,
    #[serde(default)]
    pub usage_notes: Vec<PhraseUsageNote>,
    #[serde(default)]
    pub common_mistakes: Vec<PhraseMistake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
}

// Section 3: Related - "What's connected?"

/// Variation of the phrase (regional or stylistic)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseVariation {
    pub phrase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<PhraseRegion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Related phrase with similar or opposite meaning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedPhrase {
    pub phrase: String,
    pub meaning_hint: String,
}

/// Related phrases section with variations and similar/opposite expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseSection3Related {
    #[serde(default)]
    pub variations: Vec<PhraseVariation>,
    #[serde(default)]
    pub similar_phrases: Vec<RelatedPhrase>,
    #[serde(default)]
    pub opposite_phrases: Vec<RelatedPhrase>,
    #[serde(default)]
    pub see_also: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
}

// Complete Phrase Definition

/// Complete phrase definition with all three sections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseDefinitionData {
    pub section1: PhraseSection1Overview,
    pub section2: PhraseSection2Context,
    pub section3: PhraseSection3Related,
}

// Phrase History & Saved

/// Phrase lookup history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhraseHistoryEntry {
    pub id: String,
    pub phrase: String,
    pub timestamp: i64,
    pub provider: String,
}

/// Saved phrase entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPhrase {
    pub id: String,
    pub phrase: String,
    pub phrase_type: PhraseType,
    pub tldr: String,
    pub saved_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// Detection

/// Result of input type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputType {
    Word,
    Phrase,
}

// ============================================================================
// Practice Feature Models
// ============================================================================

/// Practice mode enum matching frontend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PracticeMode {
    #[default]
    Flashcard,
    Quiz,
    FillBlank,
    Match,
}

impl std::fmt::Display for PracticeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PracticeMode::Flashcard => write!(f, "flashcard"),
            PracticeMode::Quiz => write!(f, "quiz"),
            PracticeMode::FillBlank => write!(f, "fill_blank"),
            PracticeMode::Match => write!(f, "match"),
        }
    }
}

impl std::str::FromStr for PracticeMode {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "flashcard" => Ok(PracticeMode::Flashcard),
            "quiz" => Ok(PracticeMode::Quiz),
            "fill_blank" | "fillblank" => Ok(PracticeMode::FillBlank),
            "match" => Ok(PracticeMode::Match),
            _ => Err(format!("Unknown practice mode: {}", s)),
        }
    }
}

/// Word complexity level for UI display (colored dots)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WordComplexity {
    Basic,
    Intermediate,
    Advanced,
}

impl std::fmt::Display for WordComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WordComplexity::Basic => write!(f, "basic"),
            WordComplexity::Intermediate => write!(f, "intermediate"),
            WordComplexity::Advanced => write!(f, "advanced"),
        }
    }
}

impl std::str::FromStr for WordComplexity {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" | "common" | "beginner" => Ok(WordComplexity::Basic),
            "intermediate" | "moderate" => Ok(WordComplexity::Intermediate),
            "advanced" | "complex" | "expert" => Ok(WordComplexity::Advanced),
            _ => Ok(WordComplexity::Intermediate), // Default fallback
        }
    }
}

/// Word entry for the combined Words tab (History + Saved)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordEntry {
    pub id: String,
    pub word: String,
    pub phonetic: Option<String>,
    pub short_definition: Option<String>,
    pub complexity: WordComplexity,
    pub is_idiom: bool,
    pub is_saved: bool,
    pub timestamp: i64,
    pub provider: String,
}

/// Word practice statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordPracticeStats {
    pub times_practiced: i32,
    pub times_correct: i32,
    pub last_practiced_at: Option<i64>,
    pub mastery_level: i32,
}

impl WordPracticeStats {
    pub fn accuracy(&self) -> f64 {
        if self.times_practiced == 0 {
            0.0
        } else {
            (self.times_correct as f64 / self.times_practiced as f64) * 100.0
        }
    }
}

/// Practice session record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeSession {
    pub id: i64,
    pub mode: PracticeMode,
    pub word_count: i32,
    pub correct_count: i32,
    pub duration_seconds: Option<i32>,
    pub created_at: i64,
}

impl PracticeSession {
    pub fn accuracy(&self) -> f64 {
        if self.word_count == 0 {
            0.0
        } else {
            (self.correct_count as f64 / self.word_count as f64) * 100.0
        }
    }
}

/// Individual practice result for a word in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeResult {
    pub id: i64,
    pub session_id: i64,
    pub word: String,
    pub is_correct: bool,
}

/// Daily activity for heatmap
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayActivity {
    pub date: String,
    pub words_practiced: i32,
    pub level: i32,
}

impl DayActivity {
    /// Calculate activity level (0-4) based on words practiced
    pub fn calculate_level(words_practiced: i32) -> i32 {
        match words_practiced {
            0 => 0,
            1..=5 => 1,
            6..=15 => 2,
            16..=30 => 3,
            _ => 4,
        }
    }
}

/// Practice dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeDashboard {
    pub streak: i32,
    pub total_words_practiced: i32,
    pub accuracy_rate: f64,
    pub mastered_count: i32,
    pub words_due_for_review: i32,
    pub activity_heatmap: Vec<DayActivity>,
    pub words_needing_review: Vec<WordNeedingReview>,
    pub recent_sessions: Vec<PracticeSession>,
}

/// Word flagged for review
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordNeedingReview {
    pub word: String,
    pub phonetic: Option<String>,
    pub times_missed: i32,
    pub last_practiced_at: Option<i64>,
    pub reason: ReviewReason,
}

/// Reason why a word needs review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewReason {
    LowAccuracy,
    NotRecentlyPracticed,
    NeverPracticed,
}

// Request/Response types for Tauri commands

/// Request to start a practice session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionRequest {
    pub mode: PracticeMode,
    pub words: Vec<String>,
    #[serde(default)]
    pub shuffle: bool,
}

/// Response when starting a practice session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponse {
    pub session_id: i64,
    pub words: Vec<WordForPractice>,
}

/// Request to record a practice result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordResultRequest {
    pub session_id: i64,
    pub word: String,
    pub is_correct: bool,
    #[serde(default)]
    pub response_time_ms: Option<i32>,
}

/// Individual word result in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordResult {
    pub word: String,
    pub is_correct: bool,
    pub definition: Option<String>,
}

/// Session completion summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session: PracticeSession,
    pub results: Vec<WordResult>,
}

/// Word with practice content for session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordForPractice {
    pub word: String,
    pub phonetic: Option<String>,
    pub definition: Option<String>,
    pub example: Option<String>,
    #[serde(default)]
    pub quiz_options: Vec<String>,
    pub fill_blank_sentence: Option<String>,
}

// ============================================================================
// Practice Models Tests
// ============================================================================

#[cfg(test)]
mod practice_tests {
    use super::*;
    
    #[test]
    fn test_practice_mode_serialization() {
        let mode = PracticeMode::Flashcard;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"flashcard\"");
        
        let parsed: PracticeMode = serde_json::from_str("\"quiz\"").unwrap();
        assert_eq!(parsed, PracticeMode::Quiz);
    }
    
    #[test]
    fn test_word_complexity_from_str() {
        assert_eq!("basic".parse::<WordComplexity>().unwrap(), WordComplexity::Basic);
        assert_eq!("advanced".parse::<WordComplexity>().unwrap(), WordComplexity::Advanced);
        assert_eq!("unknown".parse::<WordComplexity>().unwrap(), WordComplexity::Intermediate);
    }
    
    #[test]
    fn test_word_practice_stats_accuracy() {
        let stats = WordPracticeStats {
            times_practiced: 10,
            times_correct: 7,
            mastery_level: 7,
            last_practiced_at: Some(1234567890),
        };
        assert!((stats.accuracy() - 70.0).abs() < 0.001);
    }
    
    #[test]
    fn test_practice_session_accuracy() {
        let session = PracticeSession {
            id: 1,
            mode: PracticeMode::Quiz,
            word_count: 5,
            correct_count: 4,
            duration_seconds: Some(10),
            created_at: 1234567890,
        };
        assert!((session.accuracy() - 80.0).abs() < 0.001);
    }
}
