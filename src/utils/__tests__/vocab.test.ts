import { describe, expect, it } from 'vitest';
import { nextVocabStars, snippetsFromMeanings } from '../vocab';
import type { MeaningItem } from '../../types';

describe('vocab helpers', () => {
  it('cycles none -> 1 -> 2 -> none', () => {
    expect(nextVocabStars(null)).toBe(1);
    expect(nextVocabStars(0)).toBe(1);
    expect(nextVocabStars(1)).toBe(2);
    expect(nextVocabStars(2)).toBeNull();
  });

  it('takes up to two meanings with the first example', () => {
    const meanings: MeaningItem[] = [
      {
        number: 1,
        partOfSpeech: 'n.',
        definition: 'loop gain',
        memoryTip: '',
        examples: ['The loop gain determines stability.', 'extra'],
      },
      {
        number: 2,
        partOfSpeech: 'n.',
        definition: 'to obtain',
        memoryTip: '',
        examples: ['She gained experience.'],
      },
      {
        number: 3,
        partOfSpeech: 'v.',
        definition: 'unused',
        memoryTip: '',
        examples: ['skip'],
      },
    ];

    expect(snippetsFromMeanings(meanings, 'fallback')).toEqual([
      {
        partOfSpeech: 'n.',
        definition: 'loop gain',
        example: 'The loop gain determines stability.',
      },
      {
        partOfSpeech: 'n.',
        definition: 'to obtain',
        example: 'She gained experience.',
      },
    ]);
  });

  it('falls back to tldr when meanings are missing', () => {
    expect(snippetsFromMeanings(undefined, ' short summary ')).toEqual([
      { partOfSpeech: '', definition: 'short summary' },
    ]);
  });
});
