import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PageHintService } from '../PageHintService';

describe('PageHintService', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-09-01T12:00:00Z'));
    PageHintService.resetForTests();
  });

  afterEach(() => {
    PageHintService.resetForTests();
    vi.useRealTimers();
  });

  it('shows a hint for the first three visits', () => {
    expect(PageHintService.recordVisit('home')).toBe(true);
    expect(PageHintService.load().home.visits).toBe(1);

    vi.advanceTimersByTime(301);
    expect(PageHintService.recordVisit('home')).toBe(true);
    expect(PageHintService.load().home.visits).toBe(2);

    vi.advanceTimersByTime(301);
    expect(PageHintService.recordVisit('home')).toBe(true);
    expect(PageHintService.load().home.visits).toBe(3);

    vi.advanceTimersByTime(301);
    expect(PageHintService.recordVisit('home')).toBe(false);
    expect(PageHintService.load().home.visits).toBe(3);
    expect(PageHintService.peek('home')).toBe(false);
  });

  it('hides a hint forever after dismiss even with one visit', () => {
    expect(PageHintService.recordVisit('vocab')).toBe(true);
    PageHintService.dismiss('vocab');
    expect(PageHintService.peek('vocab')).toBe(false);

    vi.advanceTimersByTime(301);
    expect(PageHintService.recordVisit('vocab')).toBe(false);
    expect(PageHintService.load().vocab.visits).toBe(1);
  });

  it('treats corrupt JSON as an empty state', () => {
    localStorage.setItem('mellilex.pageHints', '{not json');
    expect(PageHintService.peek('settings')).toBe(true);
    expect(PageHintService.load().settings).toEqual({ visits: 0, dismissed: false });
  });

  it('does not double-count a StrictMode remount', () => {
    expect(PageHintService.recordVisit('explore')).toBe(true);
    expect(PageHintService.recordVisit('explore')).toBe(true);
    expect(PageHintService.load().explore.visits).toBe(1);
  });
});
