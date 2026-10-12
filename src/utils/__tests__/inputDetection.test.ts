import { describe, it, expect } from 'vitest';
import {
  containsHan,
  detectInputType,
  isEnglishLexeme,
  normalizePhrase,
  sanitizeDisplayText,
  truncateForSearchBar,
} from '../inputDetection';

describe('inputDetection', () => {
  describe('detectInputType', () => {
    it('should detect single words correctly', () => {
      expect(detectInputType('hello')).toBe('word');
      expect(detectInputType('  hello  ')).toBe('word');
      expect(detectInputType('HELLO')).toBe('word');
    });

    it('should detect hyphenated words as single words', () => {
      expect(detectInputType('self-esteem')).toBe('word');
      expect(detectInputType('well-known')).toBe('word');
      expect(detectInputType('mother-in-law')).toBe('word');
    });

    it('should detect phrases correctly', () => {
      expect(detectInputType('break the ice')).toBe('phrase');
      expect(detectInputType('piece of cake')).toBe('phrase');
      expect(detectInputType('look up')).toBe('phrase');
    });

    it('should handle phrases with extra whitespace', () => {
      expect(detectInputType('  break   the   ice  ')).toBe('phrase');
    });

    it('should handle empty input as word', () => {
      expect(detectInputType('')).toBe('word');
      expect(detectInputType('   ')).toBe('word');
    });

    it('should detect Chinese queries as zh-en before phrase splitting', () => {
      expect(detectInputType('餐桌')).toBe('zh-en');
      expect(detectInputType('  一次性 用品  ')).toBe('zh-en');
      expect(containsHan('餐桌')).toBe(true);
      expect(containsHan('table')).toBe(false);
      expect(isEnglishLexeme('mechanical')).toBe(true);
      expect(isEnglishLexeme('self-control')).toBe(true);
      expect(isEnglishLexeme('ad hoc')).toBe(true);
      expect(isEnglishLexeme('机械的')).toBe(false);
      expect(isEnglishLexeme('メカニカル')).toBe(false);
    });

    it('should treat mixed Chinese-English bios as translation, not zh-en', () => {
      expect(
        detectInputType(
          'Bai Yu (白宇) is a lecturer in the College of Education Sciences at HKUST(GZ). He holds a PhD in'
        )
      ).toBe('translate');
    });

    it('should treat long text as translation even without punctuation', () => {
      expect(detectInputType('a'.repeat(40))).toBe('translate');
      expect(detectInputType('a'.repeat(39))).toBe('word');
    });

    it('should treat spaced sentence punctuation as translation', () => {
      expect(detectInputType('Hello, world')).toBe('translate');
    });

    it('should keep short words with internal punctuation as words', () => {
      expect(detectInputType("don't")).toBe('word');
      expect(detectInputType('Hello!')).toBe('word');
      expect(detectInputType('gpt-5.4')).toBe('word');
    });
  });

  describe('normalizePhrase', () => {
    it('should normalize phrases correctly', () => {
      expect(normalizePhrase('  Break The ICE  ')).toBe('break the ice');
      expect(normalizePhrase('PIECE   OF   CAKE')).toBe('piece of cake');
    });
  });

  describe('sanitizeDisplayText', () => {
    it('should trim whitespace', () => {
      expect(sanitizeDisplayText('  hello  ')).toBe('hello');
    });

    it('should remove leading non-alphanumeric characters', () => {
      expect(sanitizeDisplayText('...hello')).toBe('hello');
      expect(sanitizeDisplayText('---test')).toBe('test');
    });

    it('should remove trailing non-alphanumeric characters', () => {
      expect(sanitizeDisplayText('hello...')).toBe('hello');
      expect(sanitizeDisplayText('test---')).toBe('test');
    });

    it('should handle undefined input', () => {
      expect(sanitizeDisplayText(undefined)).toBe('');
    });

    it('should preserve internal non-alphanumeric characters', () => {
      expect(sanitizeDisplayText('self-esteem')).toBe('self-esteem');
      expect(sanitizeDisplayText("don't")).toBe("don't");
    });

    it('should keep Chinese queries instead of stripping them', () => {
      expect(sanitizeDisplayText('餐桌')).toBe('餐桌');
      expect(sanitizeDisplayText('  一次性用品  ')).toBe('一次性用品');
    });
  });

  describe('truncateForSearchBar', () => {
    it('keeps short text unchanged', () => {
      expect(truncateForSearchBar('eloquent')).toBe('eloquent');
    });

    it('collapses a long query to one ellipsis line', () => {
      expect(
        truncateForSearchBar(
          'Dear Professor Albright, thank you for the letter of recommendation.'
        )
      ).toBe('Dear Professor Albri……');
    });
  });
});
