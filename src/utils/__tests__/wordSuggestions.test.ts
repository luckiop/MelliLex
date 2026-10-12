import { describe, expect, it } from 'vitest';
import { canSuggestQuery } from '../wordSuggestions';

describe('canSuggestQuery', () => {
  it('accepts partial English words', () => {
    expect(canSuggestQuery('capt')).toBe(true);
    expect(canSuggestQuery('captian')).toBe(true);
    expect(canSuggestQuery("don't")).toBe(true);
  });

  it('rejects phrases, single letters, and non-latin text', () => {
    expect(canSuggestQuery('c')).toBe(false);
    expect(canSuggestQuery('break the ice')).toBe(false);
    expect(canSuggestQuery('餐桌')).toBe(false);
    expect(canSuggestQuery('hello, world')).toBe(false);
  });
});
