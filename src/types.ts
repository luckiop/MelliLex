


export enum WordFrequency {
  VeryCommon = "VeryCommon",
  Common = "Common",
  Uncommon = "Uncommon",
  Rare = "Rare",
  VeryRare = "VeryRare",
}

// Core Data Models - Word progressive structure

export interface WordSection1Header {
  word: string;
  pronunciation: string;
  syllables: string;
  origin: string;
  formality: FormalityInfo;
  domains: string[];
  tldr: string;
}

export interface FormalityInfo {
  level: string;
  percentage: number;
}

export interface WordSection2Meanings {
  meanings: MeaningItem[];
}

export interface MeaningItem {
  number: number;
  partOfSpeech: string;
  /** Short label (2–8 CJK / 1–4 English words), not the full definition. */
  briefGloss?: string;
  definition: string;
  memoryTip: string;
  examples: string[];
}

export interface MistakeItem {
  type: string;
  incorrectUsage: string;
  correction: string;
  category: 'semantic' | 'grammatical' | 'pronunciation' | 'context';
}

export interface WordSection3Related {
  synonyms: string[];
  antonyms: string[];
  collocations: CollocationItem[];
}

export interface CollocationItem {
  phrase: string;
  example: string;
}

// Persistence Models

export interface WordHistory {
  id: string;
  word: string;
  timestamp: number;
  aiProvider: string;
}

// Settings Models

export type TypographyOption = 'modern' | 'classic' | 'friendly';

export interface AppSettings {
  aiProvider: AiProviderType;
  openAiConfig?: OpenAiConfig;
  anthropicConfig?: AnthropicConfig;
  geminiConfig?: GeminiConfig;
  ollamaConfig?: OllamaConfig;
  theme: 'light' | 'dark' | 'system';
  exportSettings?: ExportSettings;
  explanationLanguage?: string; // Language for word explanations (default: "English")
  uiLanguage?: string; // Language for UI interface (default: auto-detected)
  enableGlobalLookup: boolean; // Enable global lookup feature
  typographyMode?: TypographyOption; // Preferred typography pairing
  technicalQuery?: boolean; // Prefer CS/control/robotics sense first in meanings
  vocabNoteEnabled?: boolean;
  vocabNotePath?: string;
  vocabNoteSplitFiles?: boolean;
  devLogEnabled?: boolean;
}

export type VocabCategory = 'daily' | 'technical';

export interface VocabEntry {
  word: string;
  stars: number;
  category: VocabCategory;
  addedAt: number;
  updatedAt: number;
  shortDefinition?: string;
}

export interface VocabUpsertResult {
  entry: VocabEntry;
  noteAppended: boolean;
  noteError?: string | null;
}

export interface VocabNoteSnippet {
  partOfSpeech: string;
  briefGloss?: string;
  definition: string;
  example?: string;
}

// Must match Rust SUPPORTED_LANGUAGES in models.rs
export const SUPPORTED_LANGUAGES = [
  { name: "English", code: "en", nativeName: "English" },
  { name: "Spanish", code: "es", nativeName: "Español" },
  { name: "Portuguese", code: "pt", nativeName: "Português" },
  { name: "French", code: "fr", nativeName: "Français" },
  { name: "German", code: "de", nativeName: "Deutsch" },
  { name: "Hindi", code: "hi", nativeName: "हिन्दी" },
  { name: "Arabic", code: "ar", nativeName: "العربية" },
  { name: "Chinese (Simplified)", code: "zh", nativeName: "中文" },
  { name: "Japanese", code: "ja", nativeName: "日本語" },
  { name: "Korean", code: "ko", nativeName: "한국어" },
  { name: "Italian", code: "it", nativeName: "Italiano" },
  { name: "Turkish", code: "tr", nativeName: "Türkçe" },
  { name: "Russian", code: "ru", nativeName: "Русский" },
];

export type AiProviderType = 'openai' | 'anthropic' | 'gemini' | 'ollama';

export interface OpenAiConfig {
  apiKey: string;
  model: string;
}

export interface AnthropicConfig {
  apiKey: string;
  model: string;
}

export interface GeminiConfig {
  apiKey: string;
  model: string;
}

export interface OllamaConfig {
  endpoint: string;
  model: string;
}

export interface ExportSettings {
  includeExploration?: boolean;
  capacities?: CapacitiesConfig;
}

export interface CapacitiesConfig {
  apiToken: string;
  spaceId: string;
  defaultTags: string[];
  noTimestamp?: boolean;
}

// API Response Types

export interface SpellCheckResponse {
  originalWord: string;
  isCorrect: boolean;
  suggestedWord: string | null;
  alternatives: string[];
}

export interface AppErrorPayload {
  type: string;
  message: string;
}

// UI State Types

export type LoadingState = 'idle' | 'loading' | 'success' | 'error';

export interface AiModel {
  id: string;
  name: string;
}

// Word Exploration Types

// Must match Rust enum in models.rs
export enum FormalityLevel {
  VeryFormal = "VeryFormal",
  Formal = "Formal",
  Neutral = "Neutral",
  Informal = "Informal",
  VeryInformal = "VeryInformal",
}

export interface FormalityAlternative {
  word: string;
  level: FormalityLevel;
  context: string;
  explanation: string;
}

export interface UsagePattern {
  template: string;
  description: string;
  examples: string[];
  patternType?: string;
}

export interface DomainExploration {
  domain: string;
  usageFrequency: WordFrequency;
  commonCollocations: string[];
  examples: string[];
  isExpanded?: boolean;
}

export interface PracticeExercise {
  question: string;
  options: string[];
  correctAnswer: string;
  explanation: string;
  exerciseType: string;
  isAnswered?: boolean;
  userAnswer?: string;
}

export interface CachedFormalityData {
  formalityPercentage: number;
  formalityAlternatives: FormalityAlternative[];
}

export interface CachedPracticeData {
  practiceExercises: PracticeExercise[];
}

export interface CachedExploreFeatures {
  formality?: CachedFormalityData | null;
  domains?: DomainExploration[] | null;
  usage?: UsagePattern[] | null;
  practice?: CachedPracticeData | null;
}

export interface GlobalLookupTriggerPayload {
  source: string;
  word: string | null;
}

// Phrase Types (Multi-word expressions)

export type PhraseType = 
  | 'idiom' 
  | 'proverb' 
  | 'phrasalVerb' 
  | 'collocation' 
  | 'expression' 
  | 'saying';

export type PhraseRegion = 
  | 'universal' 
  | 'american' 
  | 'british' 
  | 'australian';

export type InputType = 'word' | 'phrase' | 'zh-en';

export interface ZhEnEquivalent {
  expression: string;
  partOfSpeech: string;
  gloss: string;
  example: string;
  occasion: string;
  contrast: string;
}

export interface ZhEnResult {
  query: string;
  equivalents: ZhEnEquivalent[];
}

// Section 1: Overview
export interface PhraseSection1Overview {
  phrase: string;
  phraseType: PhraseType;
  tldr: string;
  literalMeaning?: string;
  actualMeaning: string;
  formality: FormalityInfo;
  region: PhraseRegion;
}

// Section 2: Context
export interface PhraseOrigin {
  story: string;
  era?: string;
  source?: string;
  evolution?: string;
}

export interface PhraseUsageNote {
  context: string;
  example: string;
  tone?: string;
}

export interface PhraseMistake {
  mistakeType: string;
  incorrect: string;
  correct: string;
  explanation: string;
}

export interface PhraseSection2Context {
  origin: PhraseOrigin;
  usageNotes: PhraseUsageNote[];
  commonMistakes: PhraseMistake[];
}

// Section 3: Related
export interface PhraseVariation {
  phrase: string;
  region?: PhraseRegion;
  note?: string;
}

export interface RelatedPhrase {
  phrase: string;
  meaningHint: string;
}

export interface PhraseSection3Related {
  variations: PhraseVariation[];
  similarPhrases: RelatedPhrase[];
  oppositePhrases: RelatedPhrase[];
  seeAlso: string[];
}

// Complete Phrase Definition
export interface PhraseDefinitionData {
  section1: PhraseSection1Overview;
  section2: PhraseSection2Context;
  section3: PhraseSection3Related;
}

// Phrase Type Display Helpers
export const getPhraseTypeDisplay = (type: PhraseType): string => {
  const displayMap: Record<PhraseType, string> = {
    idiom: 'Idiom',
    proverb: 'Proverb',
    phrasalVerb: 'Phrasal Verb',
    collocation: 'Collocation',
    expression: 'Expression',
    saying: 'Saying',
  };
  return displayMap[type];
};
