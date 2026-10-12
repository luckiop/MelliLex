import { makeObservable, observable, computed, runInAction, action } from 'mobx';
import { api } from '../services/api';
import type {
  WordSection1Header,
  WordSection2Meanings,
  WordSection3Related,
} from '../types';
import type { IEventListener } from '../services/EventListenerService';
import type { ITimeoutService } from '../services/TimeoutService';
import type { ILogger } from '../services/LoggerService';
import { EVENTS, TIMEOUTS } from '../constants/events';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';
import { sanitizeDisplayText } from '../utils/inputDetection';
import { runProgressiveSearch } from './helpers/progressiveSearch';
import { BaseStore } from './BaseStore';

/**
 * ProgressiveWordStore manages progressive word definition loading.
 * 
 * Flow:
 * 1. Header (word, pronunciation, formality, TL;DR)
 * 2. Meanings (definitions with memory tips)
 * 3. Related Words (synonyms, antonyms, collocations)
 */
export class ProgressiveWordStore extends BaseStore {
  currentWord: string = '';
  private currentWordKey: string = '';
  private activeHeaderWordKey: string | null = null;
  private searchToken = 0;
  
  // Section States
  headerSection: WordSection1Header | null = null;
  meaningsSection: WordSection2Meanings | null = null;
  relatedSection: WordSection3Related | null = null;

  // Section loading flags
  hasHeaderSection: boolean = false;
  hasMeaningsSection: boolean = false;
  hasRelatedSection: boolean = false;

  constructor(
    private eventListener: IEventListener,
    private timeoutService: ITimeoutService,
    private logger: ILogger
  ) {
    super();
    makeObservable(this, {
      currentWord: observable,
      headerSection: observable,
      meaningsSection: observable,
      relatedSection: observable,
      hasHeaderSection: observable,
      hasMeaningsSection: observable,
      hasRelatedSection: observable,
      searchWord: action,
      clearSearch: action,
      clearCurrentWord: action,
      isComplete: computed,
      loadingProgress: computed,
    });
    this.setupProgressiveListeners();
  }

  private async setupProgressiveListeners() {
    this.logger.info('[ProgressiveWordStore] Setting up 3-section listeners...');

    // Header section
    await this.eventListener.listen<WordSection1Header>(
      EVENTS.WORD_SECTION_1_HEADER,
      (payload) => {
        this.logger.info(`[ProgressiveWordStore] Header section received: ${payload.word}`);
        runInAction(() => {
          const payloadWordKey = this.normalizeWord(payload.word);
          if (!payloadWordKey) {
            this.logger.info('[ProgressiveWordStore] Received header section without a word payload');
            return;
          }

          if (payloadWordKey !== this.currentWordKey) {
            this.logger.info(
              `[ProgressiveWordStore] Ignoring stale header section for: ${payload.word} (expected ${this.currentWord})`
            );
            return;
          }
          this.headerSection = payload;
          this.hasHeaderSection = true;
          this.activeHeaderWordKey = payloadWordKey;
          this.setLoading();
          this.logger.info('[ProgressiveWordStore] Header section loaded');
        });
      }
    );

    // Meanings section
    await this.eventListener.listen<WordSection2Meanings>(
      EVENTS.WORD_SECTION_2_MEANINGS,
      (payload) => {
        this.logger.info(
          `[ProgressiveWordStore] Meanings section received: ${payload.meanings?.length || 0} meanings`
        );
        runInAction(() => {
          if (!this.isActiveWordCurrent()) {
            this.logger.info('[ProgressiveWordStore] Ignoring meanings for inactive/stale word payload');
            return;
          }
          this.meaningsSection = payload;
          this.hasMeaningsSection = true;
          this.logger.info('[ProgressiveWordStore] Meanings section loaded');
          this.checkComplete();
        });
      }
    );

    // Section 3: Related Words
    await this.eventListener.listen<WordSection3Related>(
      EVENTS.WORD_SECTION_3_RELATED,
      (payload) => {
        this.logger.info(
          `[ProgressiveWordStore] Related section received: ${payload.synonyms?.length || 0} synonyms, ${payload.antonyms?.length || 0} antonyms`
        );
        runInAction(() => {
          if (!this.isActiveWordCurrent()) {
            this.logger.info('[ProgressiveWordStore] Ignoring related words for inactive/stale word payload');
            return;
          }
          this.relatedSection = payload;
          this.hasRelatedSection = true;
          this.logger.info('[ProgressiveWordStore] Related section loaded');
          this.checkComplete();
        });
      }
    );

    this.logger.info('[ProgressiveWordStore] All definition listeners registered');
  }

  async searchWord(word: string) {
    const trimmedWord = word.trim();
    const sanitizedDisplayWord = sanitizeDisplayText(trimmedWord);
    if (!sanitizedDisplayWord) {
      this.setError('Please enter a word');
      return;
    }

    const normalizedKey = this.normalizeWord(trimmedWord);
    const token = ++this.searchToken;
    this.logger.info(`[ProgressiveWordStore] Starting progressive search: ${sanitizedDisplayWord}`);

    this.resetWordTracking();
    this.currentWord = sanitizedDisplayWord;
    this.currentWordKey = normalizedKey;
    this.setLoading();
    this.headerSection = null;
    this.meaningsSection = null;
    this.relatedSection = null;
    this.hasHeaderSection = false;
    this.hasMeaningsSection = false;
    this.hasRelatedSection = false;

    await runProgressiveSearch({
      timeoutService: this.timeoutService,
      timeoutMs: TIMEOUTS.WORD_SEARCH,
      isLoading: () => this.isLoading && token === this.searchToken,
      onTimeout: () => {
        if (token !== this.searchToken) {
          return;
        }
        if (this.finishIfPrimarySectionsReady()) {
          this.logger.error(
            `[ProgressiveWordStore] Related words timed out for: ${word}; keeping header and meanings`
          );
          return;
        }
        this.setError(
          `Search timed out after ${TIMEOUTS.WORD_SEARCH / 1000} seconds. Please try again.`
        );
        this.activeHeaderWordKey = null;
        this.logger.error(`[ProgressiveWordStore] Timeout for word: ${word}`);
      },
      onError: (err) => {
        if (token !== this.searchToken) {
          return;
        }
        if (this.finishIfPrimarySectionsReady()) {
          this.logger.error(
            '[ProgressiveWordStore] Search command failed after header/meanings; keeping results',
            err as Error
          );
          return;
        }
        this.logger.error('[ProgressiveWordStore] Search failed', err as Error);
        this.setError(getErrorMessage(err, ERROR_MESSAGES.SEARCH_FAILED));
        this.activeHeaderWordKey = null;
      },
      invoke: () => api.searchWordProgressive(sanitizedDisplayWord),
      logger: this.logger,
      logPrefix: '[ProgressiveWordStore]',
    });

    if (token === this.searchToken) {
      this.finishIfPrimarySectionsReady();
    }
  }

  clearSearch() {
    this.searchToken += 1;
    this.resetWordTracking();
    this.headerSection = null;
    this.meaningsSection = null;
    this.relatedSection = null;
    this.setIdle();
    this.hasHeaderSection = false;
    this.hasMeaningsSection = false;
    this.hasRelatedSection = false;
  }

  clearCurrentWord() {
    this.currentWord = '';
  }

  cleanup() {
    this.eventListener.cleanup();
    this.timeoutService.clearTimeout();
  }

  get isComplete() {
    return this.hasHeaderSection && this.hasMeaningsSection && this.hasRelatedSection;
  }

  get loadingProgress() {
    if (!this.isLoading && !this.isSuccess) return 0;

    const sections = [
      this.hasHeaderSection,
      this.hasMeaningsSection,
      this.hasRelatedSection,
    ];
    const completed = sections.filter(Boolean).length;
    return (completed / sections.length) * 100;
  }

  /**
   * Check if all sections have loaded and transition to success state
   */
  private checkComplete(): void {
    if (this.hasHeaderSection && this.hasMeaningsSection) {
      this.setSuccess();
      this.logger.info('[ProgressiveWordStore] Header and meanings loaded — ready for another lookup');
    }
  }

  /** Header + meanings are enough to unblock the UI if related words never arrive. */
  private finishIfPrimarySectionsReady(): boolean {
    if (!this.hasHeaderSection || !this.hasMeaningsSection) {
      return false;
    }
    if (!this.hasRelatedSection) {
      this.hasRelatedSection = true;
      this.relatedSection = this.relatedSection ?? {
        synonyms: [],
        antonyms: [],
        collocations: [],
      };
    }
    this.setSuccess();
    return true;
  }

  private normalizeWord(word: string | undefined): string {
    return sanitizeDisplayText(word).toLowerCase();
  }

  private isActiveWordCurrent(): boolean {
    return Boolean(
      this.currentWordKey &&
      this.activeHeaderWordKey &&
      this.activeHeaderWordKey === this.currentWordKey
    );
  }

  private resetWordTracking(): void {
    this.currentWord = '';
    this.currentWordKey = '';
    this.activeHeaderWordKey = null;
  }
}
