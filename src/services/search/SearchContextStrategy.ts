import type { NavigateFunction } from 'react-router-dom';
import type { SearchCoordinator } from '../SearchCoordinator';
import type { ProgressiveWordStore } from '../../stores/ProgressiveWordStore';
import type { HistoryStore } from '../../stores/HistoryStore';

type SearchContextType = 'word' | 'history';

interface SearchContextStrategy {
  readonly type: SearchContextType;
  readonly placeholderKey: string;
  readonly requiresSpellCheck: boolean;
  readonly clearOnSubmit: boolean;

  isBusy(): boolean;
  getSyncedValue(): string;
  performSearch(query: string): Promise<void>;
  performSuggestion?(word: string): Promise<void>;
  handleInputChange?(value: string): void;
}

interface SearchStrategyParams {
  pathname: string;
  navigate: NavigateFunction;
  searchCoordinator: SearchCoordinator;
  progressiveWordStore: ProgressiveWordStore;
  historyStore: HistoryStore;
}

export const createSearchStrategy = (params: SearchStrategyParams): SearchContextStrategy => {
  if (params.pathname.startsWith('/history')) {
    return new HistorySearchStrategy(params.historyStore);
  }

  return new WordLookupSearchStrategy({
    navigate: params.navigate,
    searchCoordinator: params.searchCoordinator,
    progressiveWordStore: params.progressiveWordStore,
    shouldNavigateToDefine: params.pathname !== '/',
  });
};

class WordLookupSearchStrategy implements SearchContextStrategy {
  readonly type = 'word' as const;
  readonly placeholderKey = 'search.placeholder';
  readonly requiresSpellCheck = true;
  readonly clearOnSubmit = true;

  constructor(private deps: {
    navigate: NavigateFunction;
    searchCoordinator: SearchCoordinator;
    progressiveWordStore: ProgressiveWordStore;
    shouldNavigateToDefine: boolean;
  }) {}

  isBusy(): boolean {
    return this.deps.searchCoordinator.isBusy;
  }

  getSyncedValue(): string {
    return this.deps.progressiveWordStore.currentWord || '';
  }

  async performSearch(query: string): Promise<void> {
    if (this.deps.shouldNavigateToDefine) {
      this.deps.navigate('/');
    }
    await this.deps.searchCoordinator.search(query, { source: 'search-bar' });
  }

  async performSuggestion(word: string): Promise<void> {
    await this.performSearch(word);
  }

  handleInputChange(value: string): void {
    if (!value.trim()) {
      this.deps.progressiveWordStore.clearCurrentWord();
    }
  }
}

class HistorySearchStrategy implements SearchContextStrategy {
  readonly type = 'history' as const;
  readonly placeholderKey = 'history.searchPlaceholder';
  readonly requiresSpellCheck = false;
  readonly clearOnSubmit = false;

  constructor(private historyStore: HistoryStore) {}

  isBusy(): boolean {
    return this.historyStore.loadingState === 'loading';
  }

  getSyncedValue(): string {
    return this.historyStore.searchQuery;
  }

  async performSearch(query: string): Promise<void> {
    this.historyStore.setSearchQuery(query.trim());
  }

  handleInputChange(value: string): void {
    this.historyStore.setSearchQuery(value);
  }
}
