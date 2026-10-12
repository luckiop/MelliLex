export interface ValidationResult {
  valid: boolean;
  error?: string;
}

/**
 * Service for client-side only validation.
 * 
 * The Rust backend (validation.rs) handles input sanitization and validation
 * for dictionary lookups using Unicode-aware methods (char.is_alphanumeric()).
 * 
 * This frontend service ONLY validates client-side features:
 * - History search filtering
 * - Notes (future feature)
 * - Tags (future feature)
 * 
 * Word/phrase validation is handled by the backend.
 */
export class ValidationService {
  private static readonly MAX_SEARCH_QUERY_LENGTH = 8000;
  private static readonly MAX_NOTE_LENGTH = 500;
  private static readonly MAX_TAG_LENGTH = 30;

  static validateSearchQuery(query: string): ValidationResult {
    if (!query) {
      return { valid: true }; // Empty shows all results
    }

    if (query.length > this.MAX_SEARCH_QUERY_LENGTH) {
      return { valid: false, error: `Search query must be ${this.MAX_SEARCH_QUERY_LENGTH} characters or less` };
    }

    return { valid: true };
  }

  static validateNote(note: string | undefined): ValidationResult {
    if (!note) {
      return { valid: true }; // Notes are optional
    }

    if (note.length > this.MAX_NOTE_LENGTH) {
      return { valid: false, error: `Note must be ${this.MAX_NOTE_LENGTH} characters or less` };
    }

    return { valid: true };
  }

  static validateTag(tag: string): ValidationResult {
    if (!tag || !tag.trim()) {
      return { valid: false, error: 'Tag cannot be empty' };
    }

    const trimmed = tag.trim();
    
    if (trimmed.length > this.MAX_TAG_LENGTH) {
      return { valid: false, error: `Tag must be ${this.MAX_TAG_LENGTH} characters or less` };
    }

    // Tags should be simple identifiers for organization
    if (!/^[\p{L}\p{N}\s]+$/u.test(trimmed)) {
      return { valid: false, error: 'Tag can only contain letters, numbers, and spaces' };
    }

    return { valid: true };
  }
}
