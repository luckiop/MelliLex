import { makeObservable, observable, action, runInAction } from 'mobx';
import { api } from '../services/api';
import type { ZhEnResult } from '../types';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';
import { sanitizeDisplayText } from '../utils/inputDetection';
import { BaseStore } from './BaseStore';

export class ZhEnStore extends BaseStore {
  currentQuery: string = '';
  result: ZhEnResult | null = null;

  constructor() {
    super();
    makeObservable(this, {
      currentQuery: observable,
      result: observable,
      search: action,
      clearSearch: action,
    });
  }

  async search(query: string) {
    const sanitized = sanitizeDisplayText(query);
    if (!sanitized) {
      this.setError('Please enter a Chinese word or phrase');
      return;
    }

    this.currentQuery = sanitized;
    this.result = null;
    this.setLoading();

    try {
      const result = await api.searchZhEn(sanitized);
      runInAction(() => {
        this.result = result;
        this.currentQuery = result.query?.trim() || sanitized;
        this.setSuccess();
      });
    } catch (err) {
      runInAction(() => {
        this.setError(getErrorMessage(err, ERROR_MESSAGES.SEARCH_FAILED));
      });
    }
  }

  clearSearch() {
    this.currentQuery = '';
    this.result = null;
    this.setIdle();
  }
}
