import { makeObservable, observable, action, runInAction } from 'mobx';
import { api } from '../services/api';
import type { TranslateResult } from '../types';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';
import { BaseStore } from './BaseStore';

export class TranslateStore extends BaseStore {
  currentSource: string = '';
  result: TranslateResult | null = null;
  private requestId = 0;

  constructor() {
    super();
    makeObservable(this, {
      currentSource: observable,
      result: observable,
      search: action,
      clearSearch: action,
    });
  }

  async search(source: string) {
    const trimmed = source.trim();
    if (!trimmed) {
      this.setError('Please enter text to translate');
      return;
    }

    this.currentSource = trimmed;
    this.result = null;
    this.setLoading();
    const requestId = ++this.requestId;

    try {
      const result = await api.searchTranslate(trimmed);
      if (requestId !== this.requestId) {
        return;
      }
      runInAction(() => {
        this.result = result;
        this.currentSource = result.source?.trim() || trimmed;
        this.setSuccess();
      });
    } catch (err) {
      if (requestId !== this.requestId) {
        return;
      }
      runInAction(() => {
        this.setError(getErrorMessage(err, ERROR_MESSAGES.SEARCH_FAILED));
      });
    }
  }

  clearSearch() {
    this.requestId += 1;
    this.currentSource = '';
    this.result = null;
    this.setIdle();
  }
}
