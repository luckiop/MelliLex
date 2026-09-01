import { makeObservable, observable, action, runInAction } from 'mobx';
import { api } from '../services/api';
import type { VocabCategory, VocabEntry, VocabNoteSnippet } from '../types';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';
import { nextVocabStars } from '../utils/vocab';
import { BaseStore } from './BaseStore';

export class VocabStore extends BaseStore {
  items: VocabEntry[] = [];
  currentEntry: VocabEntry | null = null;
  pendingCategory: VocabCategory = 'daily';
  starFilter: number | 'all' = 'all';
  categoryFilter: VocabCategory | 'all' = 'all';
  currentWord: string = '';

  constructor() {
    super();
    makeObservable(this, {
      items: observable,
      currentEntry: observable,
      pendingCategory: observable,
      starFilter: observable,
      categoryFilter: observable,
      currentWord: observable,
      setStarFilter: action,
      setCategoryFilter: action,
    });
  }

  get activeCategory(): VocabCategory {
    return this.currentEntry?.category ?? this.pendingCategory;
  }

  setStarFilter(filter: number | 'all') {
    this.starFilter = filter;
  }

  setCategoryFilter(filter: VocabCategory | 'all') {
    this.categoryFilter = filter;
  }

  async loadForWord(word: string, defaultCategory: VocabCategory) {
    this.currentWord = word;
    try {
      const entry = await api.getVocabEntry(word);
      runInAction(() => {
        this.currentEntry = entry;
        if (!entry) {
          this.pendingCategory = defaultCategory;
        }
        this.error = null;
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.LOAD_VOCAB_FAILED);
      });
    }
  }

  async cycleStars(word: string, notes: VocabNoteSnippet[]) {
    const next = nextVocabStars(this.currentEntry?.stars);
    try {
      if (next == null) {
        await api.deleteVocabEntry(word);
        runInAction(() => {
          this.pendingCategory = this.currentEntry?.category ?? this.pendingCategory;
          this.currentEntry = null;
          this.items = this.items.filter(
            (item) => item.word.toLowerCase() !== word.toLowerCase()
          );
        });
        return;
      }

      const category = this.activeCategory;
      const entry = await api.upsertVocabEntry(word, next, category, notes);
      runInAction(() => {
        this.currentEntry = entry;
        this.upsertLocal(entry);
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.UPDATE_VOCAB_FAILED);
      });
      throw err;
    }
  }

  async toggleCategory(word: string) {
    const next: VocabCategory = this.activeCategory === 'daily' ? 'technical' : 'daily';
    if (!this.currentEntry) {
      runInAction(() => {
        this.pendingCategory = next;
      });
      return;
    }

    try {
      const entry = await api.upsertVocabEntry(word, this.currentEntry.stars, next);
      runInAction(() => {
        this.currentEntry = entry;
        this.upsertLocal(entry);
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.UPDATE_VOCAB_FAILED);
      });
      throw err;
    }
  }

  async loadList() {
    await this.executeAsync(
      () =>
        api.listVocabEntries({
          stars: this.starFilter === 'all' ? undefined : this.starFilter,
          category: this.categoryFilter === 'all' ? undefined : this.categoryFilter,
        }),
      ERROR_MESSAGES.LOAD_VOCAB_FAILED,
      (items) => {
        this.items = items;
      }
    );
  }

  async deleteWord(word: string) {
    try {
      await api.deleteVocabEntry(word);
      runInAction(() => {
        this.items = this.items.filter(
          (item) => item.word.toLowerCase() !== word.toLowerCase()
        );
        if (this.currentWord.toLowerCase() === word.toLowerCase()) {
          this.currentEntry = null;
        }
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.DELETE_ITEM_FAILED);
      });
      throw err;
    }
  }

  private upsertLocal(entry: VocabEntry) {
    const index = this.items.findIndex(
      (item) => item.word.toLowerCase() === entry.word.toLowerCase()
    );
    if (index >= 0) {
      this.items[index] = entry;
    } else {
      this.items = [entry, ...this.items];
    }
  }
}
