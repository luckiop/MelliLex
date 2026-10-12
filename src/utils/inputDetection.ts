import type { InputType } from '../types';

const HAN_RE = /[\u4e00-\u9fff]/;
/** Sentence/clause marks. Hyphens and apostrophes stay as part of words. */
const SENTENCE_PUNCT_RE = /[,.!?…;:。！？；：，、]/;
const LATIN_RE = /[A-Za-z]/;
export const TRANSLATION_CHAR_THRESHOLD = 40;
export const SEARCH_BAR_DISPLAY_CHARS = 20;

export function containsHan(input: string): boolean {
  return HAN_RE.test(input);
}

export function charCount(input: string): number {
  return [...input].length;
}

/** Related-word chips that can be looked up as English, not routed to zh-en. */
export function isEnglishLexeme(input: string): boolean {
  const value = input.trim();
  if (!value || containsHan(value)) {
    return false;
  }
  return /^[A-Za-z]+(?:[ '\-][A-Za-z]+)*$/.test(value);
}

export function detectInputType(input: string): InputType {
  const normalized = normalizeInput(input);
  if (!normalized) {
    return 'word';
  }

  if (looksLikeTranslation(normalized)) {
    return 'translate';
  }

  if (isPrimarilyChinese(normalized)) {
    return 'zh-en';
  }

  const wordCount = countWords(normalized);
  return wordCount > 1 ? 'phrase' : 'word';
}

function looksLikeTranslation(text: string): boolean {
  if (charCount(text) >= TRANSLATION_CHAR_THRESHOLD) {
    return true;
  }

  const mixedHanAndLatin = containsHan(text) && LATIN_RE.test(text);
  if (mixedHanAndLatin && charCount(text) >= 10) {
    return true;
  }

  if (!SENTENCE_PUNCT_RE.test(text)) {
    return false;
  }

  return /\s/.test(text) || charCount(text) >= 12;
}

function isPrimarilyChinese(text: string): boolean {
  if (!containsHan(text)) {
    return false;
  }
  const chars = [...text];
  const han = chars.filter((ch) => HAN_RE.test(ch)).length;
  const latin = chars.filter((ch) => LATIN_RE.test(ch)).length;
  return han >= latin;
}

function normalizeInput(input: string): string {
  return input.trim().split(/\s+/).join(' ');
}

function countWords(normalized: string): number {
  if (!normalized) return 0;
  return normalized.split(/\s+/).length;
}

export function normalizePhrase(input: string): string {
  return input.trim().split(/\s+/).join(' ').toLowerCase();
}

/**
 * Sanitize text for display by trimming whitespace and removing
 * leading/trailing punctuation. CJK queries are only trimmed so
 * Chinese reverse lookup is not stripped to an empty string.
 */
export function sanitizeDisplayText(text: string | undefined): string {
  const trimmed = (text ?? '').trim();
  if (!trimmed) {
    return '';
  }
  if (containsHan(trimmed)) {
    return trimmed;
  }
  return trimmed
    .replace(/^[^a-zA-Z0-9]+/, '')
    .replace(/[^a-zA-Z0-9]+$/, '');
}

/** One-line search-bar preview, e.g. "Dear Professor Albri……" */
export function truncateForSearchBar(text: string, maxChars = SEARCH_BAR_DISPLAY_CHARS): string {
  const compact = text.replace(/\s+/g, ' ').trim();
  if (!compact) {
    return '';
  }
  const chars = [...compact];
  if (chars.length <= maxChars) {
    return compact;
  }
  return `${chars.slice(0, maxChars).join('')}……`;
}
