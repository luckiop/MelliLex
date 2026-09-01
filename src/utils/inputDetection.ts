import type { InputType } from '../types';

const HAN_RE = /[\u4e00-\u9fff]/;

export function containsHan(input: string): boolean {
  return HAN_RE.test(input);
}

export function detectInputType(input: string): InputType {
  const normalized = normalizeInput(input);
  if (containsHan(normalized)) {
    return 'zh-en';
  }

  const wordCount = countWords(normalized);
  return wordCount > 1 ? 'phrase' : 'word';
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
