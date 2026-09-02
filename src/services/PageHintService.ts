export type PageHintId = 'home' | 'vocab' | 'settings' | 'explore';

export interface PageHintEntry {
  visits: number;
  dismissed: boolean;
}

export type PageHintState = Record<PageHintId, PageHintEntry>;

const STORAGE_KEY = 'mellilex.pageHints';
const MAX_VISITS = 3;
const STRICT_MODE_DEBOUNCE_MS = 300;
const PAGE_IDS: PageHintId[] = ['home', 'vocab', 'settings', 'explore'];

const lastRecordAt = new Map<PageHintId, number>();

function emptyEntry(): PageHintEntry {
  return { visits: 0, dismissed: false };
}

function emptyState(): PageHintState {
  return {
    home: emptyEntry(),
    vocab: emptyEntry(),
    settings: emptyEntry(),
    explore: emptyEntry(),
  };
}

function isEntry(value: unknown): value is PageHintEntry {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const entry = value as PageHintEntry;
  return typeof entry.visits === 'number' && typeof entry.dismissed === 'boolean';
}

/**
 * First-three-visits page hints, stored in localStorage.
 *
 * A hint shows while the page is not dismissed and has been entered fewer than
 * three times. Closing it hides that page's hint permanently.
 */
export class PageHintService {
  static load(): PageHintState {
    const state = emptyState();
    const raw = this.readRaw();
    if (!raw) {
      return state;
    }

    try {
      const parsed: unknown = JSON.parse(raw);
      if (typeof parsed !== 'object' || parsed === null) {
        return state;
      }
      const record = parsed as Record<string, unknown>;
      for (const id of PAGE_IDS) {
        if (isEntry(record[id])) {
          state[id] = {
            visits: Math.max(0, Math.floor(record[id].visits)),
            dismissed: record[id].dismissed,
          };
        }
      }
      return state;
    } catch {
      return emptyState();
    }
  }

  static peek(pageId: PageHintId): boolean {
    const entry = this.load()[pageId];
    return !entry.dismissed && entry.visits < MAX_VISITS;
  }

  static recordVisit(pageId: PageHintId): boolean {
    const state = this.load();
    const entry = state[pageId];
    if (entry.dismissed || entry.visits >= MAX_VISITS) {
      return false;
    }

    const now = Date.now();
    const last = lastRecordAt.get(pageId) ?? 0;
    if (now - last < STRICT_MODE_DEBOUNCE_MS) {
      return true;
    }

    lastRecordAt.set(pageId, now);
    entry.visits += 1;
    this.save(state);
    return true;
  }

  static dismiss(pageId: PageHintId): void {
    const state = this.load();
    state[pageId].dismissed = true;
    this.save(state);
  }

  static resetAll(): void {
    lastRecordAt.clear();
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {
      // ignore
    }
  }

  static resetForTests(): void {
    this.resetAll();
  }

  private static save(state: PageHintState): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch {
      // ignore quota / private-mode failures
    }
  }

  private static readRaw(): string | null {
    try {
      return localStorage.getItem(STORAGE_KEY);
    } catch {
      return null;
    }
  }
}
