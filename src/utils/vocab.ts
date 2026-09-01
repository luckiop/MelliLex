import type { MeaningItem, VocabNoteSnippet } from '../types';

export function nextVocabStars(stars: number | null | undefined): 1 | 2 | null {
  if (!stars) {
    return 1;
  }
  if (stars === 1) {
    return 2;
  }
  return null;
}

export function snippetsFromMeanings(
  meanings: MeaningItem[] | undefined,
  tldr: string
): VocabNoteSnippet[] {
  const fromMeanings = (meanings ?? [])
    .slice(0, 2)
    .map((meaning) => {
      const briefGloss = meaning.briefGloss?.trim();
      return {
        partOfSpeech: meaning.partOfSpeech ?? '',
        ...(briefGloss ? { briefGloss } : {}),
        definition: meaning.definition?.trim() ?? '',
        example: meaning.examples?.[0]?.trim() || undefined,
      };
    })
    .filter((snippet) => snippet.definition.length > 0);

  if (fromMeanings.length > 0) {
    return fromMeanings;
  }

  const fallback = tldr.trim();
  return fallback ? [{ partOfSpeech: '', definition: fallback }] : [];
}
