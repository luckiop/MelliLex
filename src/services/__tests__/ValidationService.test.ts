import { describe, it, expect } from 'vitest';
import { ValidationService } from '../ValidationService';

describe('ValidationService', () => {
  describe('validateSearchQuery', () => {
    it('should accept valid search queries', () => {
      const result = ValidationService.validateSearchQuery('search term');
      expect(result.valid).toBe(true);
    });

    it('should accept queries with any characters (for filtering)', () => {
      expect(ValidationService.validateSearchQuery('hello123').valid).toBe(true);
      expect(ValidationService.validateSearchQuery('hello@world.com').valid).toBe(true);
    });

    it('should accept empty queries (shows all results)', () => {
      const result = ValidationService.validateSearchQuery('');
      expect(result.valid).toBe(true);
    });

    it('should reject queries exceeding 8000 characters', () => {
      const longQuery = 'a'.repeat(8001);
      const result = ValidationService.validateSearchQuery(longQuery);
      expect(result.valid).toBe(false);
      expect(result.error).toBe('Search query must be 8000 characters or less');
    });

    it('should accept queries exactly 8000 characters', () => {
      const query = 'a'.repeat(8000);
      const result = ValidationService.validateSearchQuery(query);
      expect(result.valid).toBe(true);
    });
  });

  describe('validateNote', () => {
    it('should accept valid notes', () => {
      const result = ValidationService.validateNote('This is a note');
      expect(result.valid).toBe(true);
    });

    it('should accept empty notes', () => {
      const result = ValidationService.validateNote('');
      expect(result.valid).toBe(true);
    });

    it('should accept undefined notes', () => {
      const result = ValidationService.validateNote(undefined);
      expect(result.valid).toBe(true);
    });

    it('should accept notes with any characters', () => {
      const result = ValidationService.validateNote('Hello! 123 @#$ %^&*');
      expect(result.valid).toBe(true);
    });

    it('should reject notes exceeding 500 characters', () => {
      const longNote = 'a'.repeat(501);
      const result = ValidationService.validateNote(longNote);
      expect(result.valid).toBe(false);
      expect(result.error).toBe('Note must be 500 characters or less');
    });

    it('should accept notes exactly 500 characters', () => {
      const note = 'a'.repeat(500);
      const result = ValidationService.validateNote(note);
      expect(result.valid).toBe(true);
    });
  });

  describe('validateTag', () => {
    it('should accept valid tags', () => {
      const result = ValidationService.validateTag('Important');
      expect(result.valid).toBe(true);
    });

    it('should accept alphanumeric tags', () => {
      const result = ValidationService.validateTag('Tag123');
      expect(result.valid).toBe(true);
    });

    it('should accept tags with Unicode letters', () => {
      expect(ValidationService.validateTag('日本語').valid).toBe(true);
      expect(ValidationService.validateTag('café').valid).toBe(true);
    });

    it('should reject empty tags', () => {
      const result = ValidationService.validateTag('');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('Tag cannot be empty');
    });

    it('should reject tags exceeding 30 characters', () => {
      const longTag = 'a'.repeat(31);
      const result = ValidationService.validateTag(longTag);
      expect(result.valid).toBe(false);
      expect(result.error).toBe('Tag must be 30 characters or less');
    });

    it('should accept tags exactly 30 characters', () => {
      const tag = 'a'.repeat(30);
      const result = ValidationService.validateTag(tag);
      expect(result.valid).toBe(true);
    });

    it('should reject tags with special characters', () => {
      const result = ValidationService.validateTag('hello@world');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('Tag can only contain letters, numbers, and spaces');
    });

    it('should reject tags with hyphens', () => {
      const result = ValidationService.validateTag('hello-world');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('Tag can only contain letters, numbers, and spaces');
    });

    it('should accept tags with spaces', () => {
      const result = ValidationService.validateTag('Important Word');
      expect(result.valid).toBe(true);
    });
  });
});
