import React from 'react';
import { vi } from 'vitest';
import { render, type RenderOptions } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { RootStore, RootStoreContext } from '../stores/RootStore';
import '../i18n';

/**
 * Creates a mock RootStore with overridden store properties.
 * Uses a real RootStore as base and allows selective overrides.
 */
export function createMockRootStore(overrides: Record<string, unknown> = {}): RootStore {
  const store = Object.create(RootStore.prototype);
  
  // Provide minimal defaults for commonly accessed stores
  store.progressiveWordStore = {
    headerSection: null,
    meaningsSection: null,
    relatedSection: null,
    isLoading: false,
    hasError: false,
    error: null,
    currentWord: '',
    hasHeaderSection: false,
    searchWord: vi.fn(),
    clearCurrentWord: vi.fn(),
    cleanup: vi.fn(),
    ...overrides.progressiveWordStore as object,
  };
  
  store.phraseStore = {
    overviewSection: null,
    contextSection: null,
    relatedSection: null,
    isLoading: false,
    hasError: false,
    error: null,
    currentPhrase: '',
    hasOverviewSection: false,
    hasContextSection: false,
    hasRelatedSection: false,
    searchPhrase: vi.fn(),
    cleanup: vi.fn(),
    ...overrides.phraseStore as object,
  };
  
  store.searchCoordinator = {
    search: vi.fn(),
    checkSpelling: vi.fn(),
    currentInputType: 'word' as const,
    isBusy: false,
    resetToHome: vi.fn(),
    ...overrides.searchCoordinator as object,
  };
  
  store.historyStore = {
    searchQuery: '',
    loadingState: 'idle',
    setSearchQuery: vi.fn(),
    items: [],
    ...overrides.historyStore as object,
  };
  
  store.settingsStore = {
    settings: {
      aiProvider: 'openai',
      theme: 'light',
      enableGlobalLookup: false,
      technicalQuery: false,
      technicalDomain: 'computer science, control engineering, and robotics',
      clipboardClickSearch: true,
      vocabNoteEnabled: false,
      exportSettings: {},
      maxMeaningsDisplay: 5,
    },
    updateSettings: vi.fn().mockResolvedValue(undefined),
    ...overrides.settingsStore as object,
  };
  
  store.exploreStore = {
    formalityPercentage: null,
    formalityAlternatives: [],
    domainExplorations: [],
    usagePatterns: [],
    commonMistakes: [],
    practiceExercises: [],
    customExamples: [],
    customContext: '',
    cleanup: vi.fn(),
    resetAll: vi.fn(),
    ...overrides.exploreStore as object,
  };
  
  store.navigationStore = {
    ...overrides.navigationStore as object,
  };
  
  store.lastPageStore = {
    setLastPage: vi.fn(),
    page: '/',
    ...overrides.lastPageStore as object,
  };

  store.vocabStore = {
    currentEntry: null,
    pendingCategory: 'daily',
    activeCategory: 'daily',
    items: [],
    starFilter: 'all',
    categoryFilter: 'all',
    loadForWord: vi.fn().mockResolvedValue(undefined),
    cycleStars: vi.fn().mockResolvedValue(undefined),
    toggleCategory: vi.fn().mockResolvedValue(undefined),
    loadList: vi.fn().mockResolvedValue(undefined),
    deleteWord: vi.fn().mockResolvedValue(undefined),
    setStarFilter: vi.fn(),
    setCategoryFilter: vi.fn(),
    isLoading: false,
    error: null,
    ...overrides.vocabStore as object,
  };

  store.zhEnStore = {
    currentQuery: '',
    result: null,
    isLoading: false,
    hasError: false,
    error: null,
    search: vi.fn().mockResolvedValue(undefined),
    clearSearch: vi.fn(),
    ...overrides.zhEnStore as object,
  };

  store.translateStore = {
    currentSource: '',
    result: null,
    isLoading: false,
    hasError: false,
    error: null,
    search: vi.fn().mockResolvedValue(undefined),
    clearSearch: vi.fn(),
    ...overrides.translateStore as object,
  };
  
  store.cleanup = vi.fn();
  store.resetToHome = vi.fn();
  
  return store as RootStore;
}

interface TestWrapperOptions {
  store?: RootStore;
  route?: string;
}

/**
 * Renders a component wrapped with all required providers (Router, RootStore, i18n).
 */
export function renderWithProviders(
  ui: React.ReactElement,
  options: TestWrapperOptions & Omit<RenderOptions, 'wrapper'> = {}
) {
  const { store = createMockRootStore(), route = '/', ...renderOptions } = options;
  
  function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <MemoryRouter initialEntries={[route]}>
        <RootStoreContext.Provider value={store}>
          {children}
        </RootStoreContext.Provider>
      </MemoryRouter>
    );
  }
  
  return { ...render(ui, { wrapper: Wrapper, ...renderOptions }), store };
}
