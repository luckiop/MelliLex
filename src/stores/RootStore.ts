import { createContext, useContext } from 'react';
import { ProgressiveWordStore } from './ProgressiveWordStore';
import { PhraseStore } from './PhraseStore';
import { HistoryStore } from './HistoryStore';
import { SettingsStore } from './SettingsStore';
import { NavigationStore } from './NavigationStore';
import { ExploreStore } from './ExploreStore';
import { LastPageStore } from './LastPageStore';
import { VocabStore } from './VocabStore';
import { EventListenerService } from '../services/EventListenerService';
import { TimeoutService } from '../services/TimeoutService';
import { TauriLoggerService } from '../services/LoggerService';
import { SearchCoordinator } from '../services/SearchCoordinator';

export class RootStore {
  // Infrastructure services
  private progressiveWordEventListener = new EventListenerService();
  private progressiveWordTimeoutService = new TimeoutService();
  private phraseEventListener = new EventListenerService();
  private phraseTimeoutService = new TimeoutService();
  private logger = new TauriLoggerService();

  // Application stores
  progressiveWordStore: ProgressiveWordStore;
  phraseStore: PhraseStore;
  historyStore: HistoryStore;
  settingsStore: SettingsStore;
  navigationStore: NavigationStore;
  exploreStore: ExploreStore;
  lastPageStore: LastPageStore;
  vocabStore: VocabStore;
  
  // Service coordinators
  searchCoordinator: SearchCoordinator;

  constructor() {
    this.navigationStore = new NavigationStore();
    this.historyStore = new HistoryStore();
    this.settingsStore = new SettingsStore();
    this.lastPageStore = new LastPageStore();
    this.vocabStore = new VocabStore();
    this.settingsStore.loadSettings();

    this.progressiveWordStore = new ProgressiveWordStore(
      this.progressiveWordEventListener,
      this.progressiveWordTimeoutService,
      this.logger
    );

    this.phraseStore = new PhraseStore(
      this.phraseEventListener,
      this.phraseTimeoutService,
      this.logger
    );

    this.exploreStore = new ExploreStore(this.logger, this.progressiveWordStore);
    
    this.searchCoordinator = new SearchCoordinator(
      this.progressiveWordStore,
      this.phraseStore,
      this.historyStore,
      this.navigationStore
    );
  }

  cleanup() {
    this.progressiveWordStore.cleanup();
    this.phraseStore.cleanup();
    this.exploreStore.cleanup();
  }
}

export const RootStoreContext = createContext<RootStore | null>(null);

export const useStores = () => {
  const store = useContext(RootStoreContext);
  if (!store) {
    throw new Error('useStores must be used within a RootStoreProvider');
  }
  return store;
};
