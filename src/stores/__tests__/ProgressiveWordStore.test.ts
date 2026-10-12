import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ProgressiveWordStore } from '../ProgressiveWordStore';
import { MockEventListener } from '../../services/EventListenerService';
import { MockTimeoutService } from '../../services/TimeoutService';
import { EVENTS } from '../../constants/events';
import type { ILogger } from '../../services/LoggerService';
import type { WordSection1Header, WordSection2Meanings, WordSection3Related } from '../../types';

const apiMocks = vi.hoisted(() => ({
  searchWordProgressive: vi.fn(),
}));

vi.mock('../../services/api', () => ({
  api: apiMocks,
}));

class TestLogger implements ILogger {
  info = vi.fn();
  error = vi.fn();
  debug = vi.fn();
}

const headerPayload = (word: string): WordSection1Header => ({
  word,
  pronunciation: `/${word}/`,
  syllables: word,
  origin: 'latin',
  formality: { level: 'Neutral', percentage: 50 },
  domains: [],
  tldr: `${word} summary`,
});

const meaningsPayload = (word: string): WordSection2Meanings => ({
  meanings: [
    {
      number: 1,
      partOfSpeech: 'noun',
      definition: `${word} definition`,
      memoryTip: `${word} memory`,
      examples: [`Example using ${word}`],
    },
  ],
});

const relatedPayload = (): WordSection3Related => ({
  synonyms: ['ally'],
  antonyms: ['opponent'],
  collocations: [],
});

const flushAsync = () => new Promise((resolve) => setTimeout(resolve, 0));

describe('ProgressiveWordStore', () => {
  let listener: MockEventListener;
  let timeouts: MockTimeoutService;
  let logger: TestLogger;
  let store: ProgressiveWordStore;

  const setupStore = async () => {
    listener = new MockEventListener();
    timeouts = new MockTimeoutService();
    logger = new TestLogger();
    store = new ProgressiveWordStore(listener, timeouts, logger);
    await flushAsync();
  };

  beforeEach(async () => {
    vi.useRealTimers();
    vi.clearAllMocks();
    await setupStore();
  });

  it('sanitizes user input so backend events are not treated as stale', async () => {
    apiMocks.searchWordProgressive.mockResolvedValue(undefined);

    await store.searchWord("  'Spark  ");
    listener.emit(EVENTS.WORD_SECTION_1_HEADER, headerPayload('Spark'));

    expect(apiMocks.searchWordProgressive).toHaveBeenCalledWith('Spark');
    expect(store.headerSection?.word).toBe('Spark');
    expect(store.hasHeaderSection).toBe(true);
  });

  it('ignores section events from previous searches once a new search starts', async () => {
    apiMocks.searchWordProgressive.mockResolvedValue(undefined);

    await store.searchWord('spark');
    listener.emit(EVENTS.WORD_SECTION_1_HEADER, headerPayload('spark'));

    await store.searchWord('ember');
    listener.emit(EVENTS.WORD_SECTION_2_MEANINGS, meaningsPayload('spark'));
    listener.emit(EVENTS.WORD_SECTION_3_RELATED, relatedPayload());

    expect(store.meaningsSection).toBeNull();
    expect(store.relatedSection).toBeNull();

    listener.emit(EVENTS.WORD_SECTION_1_HEADER, headerPayload('ember'));
    listener.emit(EVENTS.WORD_SECTION_2_MEANINGS, meaningsPayload('ember'));
    listener.emit(EVENTS.WORD_SECTION_3_RELATED, relatedPayload());

    expect(store.meaningsSection?.meanings[0].definition).toContain('ember');
    expect(store.relatedSection?.synonyms).toContain('ally');
  });

  it('unblocks after header and meanings even if related words never arrive', async () => {
    apiMocks.searchWordProgressive.mockResolvedValue(undefined);

    await store.searchWord('spark');
    listener.emit(EVENTS.WORD_SECTION_1_HEADER, headerPayload('spark'));
    listener.emit(EVENTS.WORD_SECTION_2_MEANINGS, meaningsPayload('spark'));

    expect(store.hasError).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.headerSection?.word).toBe('spark');
  });

  it('rejects searches that normalize to an empty string', async () => {
    await store.searchWord("!!!");

    expect(apiMocks.searchWordProgressive).not.toHaveBeenCalled();
    expect(store.hasError).toBe(true);
    expect(store.error).toBe('Please enter a word');
  });

  it('clears the active word when a search fails before any sections arrive', async () => {
    apiMocks.searchWordProgressive.mockRejectedValueOnce(new Error('network'));

    await store.searchWord('locks');

    expect(store.hasHeaderSection).toBe(false);
    expect(store.hasError).toBe(true);
    expect(store.currentWord).toBe('locks');
  });
});
