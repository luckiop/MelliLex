import { invoke } from '@tauri-apps/api/core';
import { logger } from '../utils/logger';
import { parseError } from '../utils/errorHandler';
import type {
  WordHistory,
  AppSettings,
  AiModel,
  CachedExploreFeatures,
  SpellCheckResponse,
  VocabEntry,
  VocabNoteSnippet,
} from '../types';

const wrapInvoke = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    // Log raw error for debugging
    console.error('[wrapInvoke] Raw error from Tauri:', JSON.stringify(error, null, 2));
    console.error('[wrapInvoke] Error type:', typeof error);
    console.error('[wrapInvoke] Error keys:', error && typeof error === 'object' ? Object.keys(error) : 'N/A');
    throw parseError(error);
  }
};

// Tauri Command Wrappers

export const api = {
  // Spell Check (before word search)
  async checkSpelling(word: string): Promise<SpellCheckResponse> {
    logger.info(`[API] Checking spelling: ${word}`);
    const result = await wrapInvoke<SpellCheckResponse>('check_spelling', { word });
    logger.info(`[API] Spell check complete: ${result.isCorrect ? 'correct' : 'misspelled'}`);
    return result;
  },

  // Get word variations for highlighting (e.g., "lament" -> ["lament", "lamentation", "lamenting"])
  async getWordVariations(word: string): Promise<string[]> {
    logger.info(`[API] Getting word variations: ${word}`);
    const variations = await wrapInvoke<string[]>('get_word_variations', { word });
    logger.info(`[API] Got ${variations.length} variations for: ${word}`);
    return variations;
  },

  // Progressive Word Search (with events)
  async searchWordProgressive(word: string): Promise<void> {
    logger.info(`[API] Invoking search_word_progressive: ${word}`);
    await wrapInvoke('search_word_progressive', { word });
    logger.info('[API] search_word_progressive invoked (backend processing...)');
  },

  // Progressive Phrase Search (with events)
  async searchPhraseProgressive(phrase: string): Promise<void> {
    logger.info(`[API] Invoking search_phrase_progressive: ${phrase}`);
    await wrapInvoke('search_phrase_progressive', { phrase });
    logger.info('[API] search_phrase_progressive invoked (backend processing...)');
  },

  // History
  async getHistory(limit?: number): Promise<WordHistory[]> {
  return await wrapInvoke<WordHistory[]>('get_history', { limit });
  },

  async clearHistory(): Promise<void> {
  return await wrapInvoke('clear_history');
  },

  async deleteHistoryItem(id: string): Promise<void> {
  return await wrapInvoke('delete_history_item', { id });
  },

  async getVocabEntry(word: string): Promise<VocabEntry | null> {
    return await wrapInvoke<VocabEntry | null>('get_vocab_entry', { word });
  },

  async listVocabEntries(filters?: {
    stars?: number;
    category?: string;
  }): Promise<VocabEntry[]> {
    return await wrapInvoke<VocabEntry[]>('list_vocab_entries', {
      stars: filters?.stars,
      category: filters?.category,
    });
  },

  async upsertVocabEntry(
    word: string,
    stars: number,
    category: string,
    notes?: VocabNoteSnippet[]
  ): Promise<VocabEntry> {
    return await wrapInvoke<VocabEntry>('upsert_vocab_entry', {
      word,
      stars,
      category,
      notes,
    });
  },

  async deleteVocabEntry(word: string): Promise<void> {
    return await wrapInvoke('delete_vocab_entry', { word });
  },

  // Settings
  async getSettings(): Promise<AppSettings> {
  return await wrapInvoke<AppSettings>('get_settings');
  },

  async updateSettings(settings: AppSettings): Promise<void> {
  return await wrapInvoke('update_settings', { settings });
  },

  async updateAiProvider(provider: string, config: any): Promise<void> {
  return await wrapInvoke('update_ai_provider', { provider, config });
  },

  // Ollama
  async detectOllama(): Promise<boolean> {
  return await wrapInvoke<boolean>('detect_ollama');
  },

  async listOllamaModels(): Promise<string[]> {
  return await wrapInvoke<string[]>('list_ollama_models');
  },

  // Model Management
  async fetchAvailableModels(provider: string, apiKey: string): Promise<AiModel[]> {
  return await wrapInvoke<AiModel[]>('fetch_available_models', { provider, apiKey });
  },

  async testApiKey(provider: string, apiKey: string): Promise<boolean> {
  return await wrapInvoke<boolean>('test_api_key', { provider, apiKey });
  },

  // Utility
  async ping(): Promise<string> {
    return await invoke<string>('ping');
  },

  // Export
  async exportMarkdown(word: string, provider: string, includeTimestamp: boolean): Promise<string> {
    return await wrapInvoke<string>('export_markdown_file', { word, provider, includeTimestamp });
  },

  async exportPhraseMarkdown(phrase: string, provider: string, includeTimestamp: boolean): Promise<string> {
    return await wrapInvoke<string>('export_phrase_markdown_file', { phrase, provider, includeTimestamp });
  },

  // Cache Management
  async clearAllCache(): Promise<void> {
    return await wrapInvoke('clear_all_cache');
  },

  async clearDefinitionCache(): Promise<void> {
    return await wrapInvoke('clear_definition_cache');
  },

  async clearExplorationCache(): Promise<void> {
    return await wrapInvoke('clear_exploration_cache');
  },

  async generateContextualExamples(word: string, context: string): Promise<string[]> {
    logger.info(`[API] Generating contextual examples: ${word} in ${context}`);
    return await wrapInvoke('generate_contextual_examples', { word, context });
  },

  // Individual on-demand exploration features
  async generateFormalityAnalysis(word: string): Promise<{
    formalityPercentage: number;
    formalityAlternatives: import('../types').FormalityAlternative[];
  }> {
    logger.info(`[API] Generating formality analysis for: ${word}`);
    const result = await wrapInvoke<[number, import('../types').FormalityAlternative[]]>(
      'generate_formality_analysis',
      { word }
    );
    return {
      formalityPercentage: result[0],
      formalityAlternatives: result[1],
    };
  },

  async generateDomainExploration(
    word: string
  ): Promise<import('../types').DomainExploration[]> {
    logger.info(`[API] Generating domain exploration for: ${word}`);
    return await wrapInvoke('generate_domain_exploration', { word });
  },

  async generateUsagePatterns(word: string): Promise<import('../types').UsagePattern[]> {
    logger.info(`[API] Generating usage patterns for: ${word}`);
    return await wrapInvoke('generate_usage_patterns', { word });
  },

  async generatePracticeExercisesOnly(word: string, count: number, force = false): Promise<{
    practiceExercises: import('../types').PracticeExercise[];
  }> {
    logger.info(`[API] Generating practice exercises for: ${word}`);
    const result = await wrapInvoke<import('../types').PracticeExercise[]>(
      'generate_practice_exercises_only',
      { word, count, force }
    );
    return {
      practiceExercises: result,
    };
  },

  async generateCommonMistakes(word: string, force = false): Promise<import('../types').MistakeItem[]> {
    logger.info(`[API] Generating common mistakes for: ${word}`);
    return await wrapInvoke<import('../types').MistakeItem[]>(
      'generate_common_mistakes',
      { word, force }
    );
  },

  async getCachedExplorationFeatures(word: string): Promise<CachedExploreFeatures> {
    logger.info(`[API] Fetching cached exploration features for: ${word}`);
    return await wrapInvoke<CachedExploreFeatures>('get_cached_exploration_features', { word });
  },

  async exportToCapacities(
    apiToken: string,
    spaceId: string,
    markdown: string,
    noTimestamp: boolean
  ): Promise<void> {
    logger.info('[API] Exporting to Capacities');
    logger.debug(`[API] Export params: spaceId=${spaceId}, markdownLength=${markdown.length}, noTimestamp=${noTimestamp}`);
    
    try {
      await wrapInvoke<void>('export_to_capacities', {
        apiToken,
        spaceId,
        markdown,
        noTimestamp,
      });
      logger.info('[API] Capacities export successful');
    } catch (error) {
      const parsed = parseError(error);
      logger.error(`[API] Capacities export failed: [${parsed.type}] ${parsed.message}`);
      console.error('[API] Full error object:', error);
      throw error;
    }
  },

  // Global Lookup
  async enableGlobalLookup(): Promise<void> {
    logger.info('[API] Enabling global lookup');
    await wrapInvoke('enable_global_lookup');
    logger.info('[API] Global lookup enabled');
  },

  async disableGlobalLookup(): Promise<void> {
    logger.info('[API] Disabling global lookup');
    await wrapInvoke('disable_global_lookup');
    logger.info('[API] Global lookup disabled');
  },

  // Updates
  async checkForAppUpdates(): Promise<boolean> {
    logger.info('[API] Checking for application updates');
    const hasUpdate = await wrapInvoke<boolean>('check_for_app_updates');
    logger.info(`[API] Update check complete: ${hasUpdate ? 'update available' : 'up to date'}`);
    return hasUpdate;
  },

  // Platform Detection
  async isStoreVersion(): Promise<boolean> {
    logger.info('[API] Checking if Store version');
    const isStore = await wrapInvoke<boolean>('is_store_version');
    logger.info(`[API] Platform detection: ${isStore ? 'Microsoft Store' : 'Direct download'}`);
    return isStore;
  },
};
