import { makeObservable, observable, computed, runInAction, action } from 'mobx';
import { api } from '../services/api';
import type {
  PhraseSection1Overview,
  PhraseSection2Context,
  PhraseSection3Related,
  PhraseDefinitionData,
} from '../types';
import type { IEventListener } from '../services/EventListenerService';
import type { ITimeoutService } from '../services/TimeoutService';
import type { ILogger } from '../services/LoggerService';
import { EVENTS, TIMEOUTS } from '../constants/events';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';
import { normalizePhrase as normalizePhraseKey, sanitizeDisplayText } from '../utils/inputDetection';
import { runProgressiveSearch } from './helpers/progressiveSearch';
import { BaseStore } from './BaseStore';

/**
 * PhraseStore manages progressive phrase definition loading.
 * 
 * Flow:
 * 1. Overview (phrase type, meaning, formality, region)
 * 2. Context (origin story, usage notes, common mistakes)
 * 3. Related (variations, similar/opposite phrases)
 */
export class PhraseStore extends BaseStore {
  currentPhrase: string = '';
  private currentPhraseKey: string = '';
  private activeOverviewPhraseKey: string | null = null;
  private searchToken = 0;
  
  // Section States
  overviewSection: PhraseSection1Overview | null = null;
  contextSection: PhraseSection2Context | null = null;
  relatedSection: PhraseSection3Related | null = null;

  // Section loading flags
  hasOverviewSection: boolean = false;
  hasContextSection: boolean = false;
  hasRelatedSection: boolean = false;

  constructor(
    private eventListener: IEventListener,
    private timeoutService: ITimeoutService,
    private logger: ILogger
  ) {
    super();
    makeObservable(this, {
      currentPhrase: observable,
      overviewSection: observable,
      contextSection: observable,
      relatedSection: observable,
      hasOverviewSection: observable,
      hasContextSection: observable,
      hasRelatedSection: observable,
      searchPhrase: action,
      clearSearch: action,
      isComplete: computed,
      loadingProgress: computed,
    });
    this.setupProgressiveListeners();
  }

  private async setupProgressiveListeners() {
    this.logger.info('[PhraseStore] Setting up 3-section listeners...');

    // Section 1: Overview
    await this.eventListener.listen<PhraseSection1Overview>(
      EVENTS.PHRASE_SECTION_1_OVERVIEW,
      (payload) => {
        this.logger.info(`[PhraseStore] Overview section received: ${payload.phrase}`);
        runInAction(() => {
          const payloadPhraseKey = this.normalizePhrase(payload.phrase);
          if (!payloadPhraseKey) {
            this.logger.info('[PhraseStore] Received overview section without a phrase payload');
            return;
          }

          // For phrases, accept the AI's corrected phrase as canonical
          // (e.g., user types "see forrest for trees" -> AI returns "can't see the forest for the trees")
          // Only accept if we're currently loading (waiting for a response)
          if (!this.isLoading) {
            this.logger.info('[PhraseStore] Ignoring overview section - not loading');
            return;
          }

          // Update the canonical phrase to match what the AI returned
          this.currentPhrase = payload.phrase;
          this.currentPhraseKey = payloadPhraseKey;
          
          this.overviewSection = payload;
          this.hasOverviewSection = true;
          this.activeOverviewPhraseKey = payloadPhraseKey;
          this.setLoading();
          this.logger.info('[PhraseStore] Overview section loaded');
        });
      }
    );

    // Section 2: Context
    await this.eventListener.listen<PhraseSection2Context>(
      EVENTS.PHRASE_SECTION_2_CONTEXT,
      (payload) => {
        this.logger.info(
          `[PhraseStore] Context section received: ${payload.usageNotes?.length || 0} usage notes`
        );
        runInAction(() => {
          if (!this.isActivePhraseCurrent()) {
            this.logger.info('[PhraseStore] Ignoring context for inactive/stale phrase payload');
            return;
          }
          this.contextSection = payload;
          this.hasContextSection = true;
          this.logger.info('[PhraseStore] Context section loaded');
          this.checkCompletion();
        });
      }
    );

    // Section 3: Related
    await this.eventListener.listen<PhraseSection3Related>(
      EVENTS.PHRASE_SECTION_3_RELATED,
      (payload) => {
        this.logger.info(
          `[PhraseStore] Related section received: ${payload.similarPhrases?.length || 0} similar, ${payload.oppositePhrases?.length || 0} opposite`
        );
        runInAction(() => {
          if (!this.isActivePhraseCurrent()) {
            this.logger.info('[PhraseStore] Ignoring related phrases for inactive/stale phrase payload');
            return;
          }
          this.relatedSection = payload;
          this.hasRelatedSection = true;
          this.logger.info('[PhraseStore] Related section loaded');
          this.checkCompletion();
        });
      }
    );

    this.logger.info('[PhraseStore] All phrase listeners registered');
  }

  private checkCompletion(): void {
    if (this.hasOverviewSection && this.hasContextSection) {
      this.setSuccess();
      this.logger.info('[PhraseStore] Overview and context loaded — ready for another lookup');
    }
  }

  async searchPhrase(phrase: string) {
    const trimmedPhrase = phrase.trim();
    const sanitizedDisplayPhrase = sanitizeDisplayText(trimmedPhrase);
    if (!sanitizedDisplayPhrase) {
      this.setError('Please enter a phrase');
      return;
    }

    const normalizedKey = this.normalizePhrase(sanitizedDisplayPhrase);
    const token = ++this.searchToken;
    this.logger.info(`[PhraseStore] Starting progressive search: ${sanitizedDisplayPhrase}`);

    this.resetPhraseTracking();
    this.currentPhrase = sanitizedDisplayPhrase;
    this.currentPhraseKey = normalizedKey;
    this.setLoading();
    this.overviewSection = null;
    this.contextSection = null;
    this.relatedSection = null;
    this.hasOverviewSection = false;
    this.hasContextSection = false;
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
            `[PhraseStore] Related phrases timed out for: ${phrase}; keeping overview and context`
          );
          return;
        }
        this.setError(
          `Search timed out after ${TIMEOUTS.WORD_SEARCH / 1000} seconds. Please try again.`
        );
        this.activeOverviewPhraseKey = null;
        this.logger.error(`[PhraseStore] Timeout for phrase: ${phrase}`);
      },
      onError: (err) => {
        if (token !== this.searchToken) {
          return;
        }
        if (this.finishIfPrimarySectionsReady()) {
          this.logger.error(
            '[PhraseStore] Search command failed after overview/context; keeping results',
            err as Error
          );
          return;
        }
        this.logger.error('[PhraseStore] Search failed', err as Error);
        this.setError(getErrorMessage(err, ERROR_MESSAGES.SEARCH_FAILED));
        this.activeOverviewPhraseKey = null;
      },
      invoke: () => api.searchPhraseProgressive(sanitizedDisplayPhrase),
      logger: this.logger,
      logPrefix: '[PhraseStore]',
    });

    if (token === this.searchToken) {
      this.finishIfPrimarySectionsReady();
    }
  }

  clearSearch() {
    this.searchToken += 1;
    this.resetPhraseTracking();
    this.overviewSection = null;
    this.contextSection = null;
    this.relatedSection = null;
    this.setIdle();
    this.hasOverviewSection = false;
    this.hasContextSection = false;
    this.hasRelatedSection = false;
  }

  cleanup() {
    this.eventListener.cleanup();
    this.timeoutService.clearTimeout();
  }

  get isComplete() {
    return this.hasOverviewSection && this.hasContextSection && this.hasRelatedSection;
  }

  get loadingProgress() {
    if (!this.isLoading && !this.isSuccess) return 0;

    const sections = [
      this.hasOverviewSection,
      this.hasContextSection,
      this.hasRelatedSection,
    ];
    const completed = sections.filter(Boolean).length;
    return (completed / sections.length) * 100;
  }

  /**
   * Get complete phrase definition data for export
   */
  getPhraseDefinitionData(): PhraseDefinitionData | null {
    if (!this.overviewSection || !this.contextSection || !this.relatedSection) {
      return null;
    }
    return {
      section1: this.overviewSection,
      section2: this.contextSection,
      section3: this.relatedSection,
    };
  }

  private normalizePhrase(phrase: string | undefined): string {
    return normalizePhraseKey(sanitizeDisplayText(phrase));
  }

  private finishIfPrimarySectionsReady(): boolean {
    if (!this.hasOverviewSection || !this.hasContextSection) {
      return false;
    }
    if (!this.hasRelatedSection) {
      this.hasRelatedSection = true;
      this.relatedSection = this.relatedSection ?? {
        variations: [],
        similarPhrases: [],
        oppositePhrases: [],
        seeAlso: [],
      };
    }
    this.setSuccess();
    return true;
  }

  private isActivePhraseCurrent(): boolean {
    return Boolean(
      this.currentPhraseKey &&
      this.activeOverviewPhraseKey &&
      this.activeOverviewPhraseKey === this.currentPhraseKey
    );
  }

  private resetPhraseTracking(): void {
    this.currentPhrase = '';
    this.currentPhraseKey = '';
    this.activeOverviewPhraseKey = null;
  }
}
