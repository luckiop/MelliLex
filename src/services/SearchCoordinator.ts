import type { ProgressiveWordStore } from '../stores/ProgressiveWordStore';
import type { PhraseStore } from '../stores/PhraseStore';
import type { ZhEnStore } from '../stores/ZhEnStore';
import type { HistoryStore } from '../stores/HistoryStore';
import type { NavigationStore } from '../stores/NavigationStore';
import type { SpellCheckResponse, InputType } from '../types';
import { api } from './api';
import { containsHan, detectInputType } from '../utils/inputDetection';

export interface SearchOptions {
  source?: string;
  skipSpellCheck?: boolean;
}

export interface SearchResult {
  inputType: InputType;
  term: string;
}

/**
 * SearchCoordinator orchestrates all search-related side effects.
 * 
 * Responsibilities:
 * - Detect input type (word vs phrase)
 * - Coordinate progressive word/phrase search
 * - Add searches to history
 * - Handle navigation to Define page
 * 
 * Benefits:
 * - Single Responsibility: Components don't need to know about coordination
 * - Testability: Easy to test orchestration logic in isolation
 * - Maintainability: Changes to search flow in one place
 * - Extensibility: Easy to add new side effects (analytics, recent searches)
 * 
 * @example
 * ```tsx
 * const { searchCoordinator } = useStores();
 * await searchCoordinator.search('ephemeral');       // Single word
 * await searchCoordinator.search('break the ice');   // Phrase
 * ```
 */
export class SearchCoordinator {
  private searchInProgress = false;
  private _currentInputType: InputType = 'word';

  constructor(
    private progressiveWordStore: ProgressiveWordStore,
    private phraseStore: PhraseStore,
    private zhEnStore: ZhEnStore,
    private historyStore: HistoryStore,
    private navigationStore: NavigationStore
  ) {}



  get currentInputType(): InputType {
    return this._currentInputType;
  }

  get isBusy(): boolean {
    return (
      this.searchInProgress ||
      this.progressiveWordStore.isLoading ||
      this.phraseStore.isLoading ||
      this.zhEnStore.isLoading
    );
  }

  /**
   * Check spelling before search. Returns spell check data if misspelled.
   * 
   * @param word - Word to check
   * @returns SpellCheckResponse if misspelled, null if correct
   */
  async checkSpelling(word: string): Promise<SpellCheckResponse | null> {
    const trimmedWord = word.trim();
    if (!trimmedWord || containsHan(trimmedWord)) {
      return null;
    }

    try {
      const result = await api.checkSpelling(trimmedWord);
      
      // Return spell check data only if word is misspelled
      if (!result.isCorrect) {
        return result;
      }
      
      return null;
    } catch (error) {
      console.error('[SearchCoordinator] Spell check failed:', error);
      // On spell check failure, proceed with search anyway
      return null;
    }
  }

  /**
   * Execute a search with all side effects:
   * 1. Detect input type (word vs phrase)
   * 2. Progressive word/phrase definition lookup
   * 3. Add to search history
   * 
   * Note: Navigation to Define page happens automatically via SearchBar's
   * useNavigate hook, keeping routing concerns in UI layer.
   * 
   * @param input - Word or phrase to search for
   * @param options.skipSpellCheck - Skip spell checking (when user selects from dialog)
   */
  async search(input: string, options?: SearchOptions): Promise<SearchResult | void> {
    const trimmedInput = input.trim();
    if (!trimmedInput) {
      return;
    }

    const inputType = detectInputType(trimmedInput);
    this._currentInputType = inputType;

    const handler =
      inputType === 'zh-en'
        ? {
            store: this.zhEnStore,
            clearOther: () => {
              this.progressiveWordStore.clearSearch();
              this.phraseStore.clearSearch();
            },
            runSearch: () => this.zhEnStore.search(trimmedInput),
          }
        : inputType === 'phrase'
          ? {
              store: this.phraseStore,
              clearOther: () => {
                this.progressiveWordStore.clearSearch();
                this.zhEnStore.clearSearch();
              },
              runSearch: () => this.phraseStore.searchPhrase(trimmedInput),
            }
          : {
              store: this.progressiveWordStore,
              clearOther: () => {
                this.phraseStore.clearSearch();
                this.zhEnStore.clearSearch();
              },
              runSearch: () => this.progressiveWordStore.searchWord(trimmedInput),
            };

    if (this.searchInProgress || handler.store.isLoading) {
      console.info(
        '[SearchCoordinator] Ignoring search while another operation is in progress'
      );
      return;
    }

    this.searchInProgress = true;

    try {
      handler.clearOther();
      await handler.runSearch();
      await this.historyStore.addToHistory(trimmedInput);
      this.navigationStore.navigateTo(trimmedInput, options?.source);
      return { inputType, term: trimmedInput };
    } finally {
      this.searchInProgress = false;
    }
  }
}
