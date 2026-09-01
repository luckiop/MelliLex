import type { AppErrorPayload } from '../types';

/**
 * Centralized error handling utility.
 * Provides consistent error parsing and formatting across the application.
 */

/**
 * Error types that indicate provider configuration issues
 */
const PROVIDER_CONFIG_ERROR_TYPES = [
  'ProviderNotConfigured',
  'Config',
] as const;

/**
 * Error message patterns that indicate provider/connection issues
 */
const PROVIDER_ERROR_PATTERNS = [
  /localhost:\d+\/api\/lookup/i,          // Local API not running
  /error sending request/i,               // Network request failed
  /api key/i,                             // API key issues
  /unauthorized/i,                        // Auth issues
  /authentication/i,                      // Auth issues
  /invalid.*key/i,                        // Invalid API key
  /provider.*not.*configured/i,           // Provider not set up
  /connection refused/i,                  // Service not running
  /ECONNREFUSED/i,                        // Connection refused
] as const;

/**
 * Parse an error from various sources into a consistent format.
 * Handles Tauri error payloads, Error objects, and unknown errors.
 */
export const parseError = (error: unknown): AppErrorPayload => {
  // Handle Tauri error format with payload
  if (typeof error === 'object' && error !== null) {
    const payload = (error as any).payload ?? error;
    
    if (typeof payload.type === 'string') {
      return {
        type: payload.type,
        message: payload.message ?? 'Unknown error'
      };
    }
    
    if (typeof payload.message === 'string') {
      return {
        type: 'Unknown',
        message: payload.message
      };
    }
  }

  // Handle standard Error objects
  if (error instanceof Error) {
    return {
      type: 'Unknown',
      message: error.message
    };
  }

  // Fallback for unknown error types
  return {
    type: 'Unknown',
    message: 'An unexpected error occurred'
  };
};

/**
 * Extract user-friendly error message with fallback.
 */
export const getErrorMessage = (error: unknown, fallback: string): string => {
  const parsed = parseError(error);
  
  if (PROVIDER_CONFIG_ERROR_TYPES.includes(parsed.type as any)) {
    return getProviderConfigErrorMessage(error);
  }
  
  return parsed.message || fallback;
};

/**
 * Check if error indicates a provider configuration issue.
 * This includes API key problems, provider not set up, or connection failures.
 */
export const isProviderConfigError = (error: unknown): boolean => {
  const parsed = parseError(error);
  
  // Check error type
  if (PROVIDER_CONFIG_ERROR_TYPES.includes(parsed.type as any)) {
    return true;
  }
  
  // Check error message patterns
  const message = parsed.message || '';
  return PROVIDER_ERROR_PATTERNS.some(pattern => pattern.test(message));
};

/**
 * Get a user-friendly message for provider configuration errors.
 */
export const getProviderConfigErrorMessage = (error: unknown): string => {
  const parsed = parseError(error);
  const message = parsed.message || '';
  
  // Specific error type messages
  if (parsed.type === 'ProviderNotConfigured') {
    return `AI provider is not configured. Please set up your API key in Settings.`;
  }
  
  // Pattern-based messages
  if (/localhost:\d+\/api\/lookup/i.test(message) || /connection refused/i.test(message)) {
    return 'Could not connect to AI service. Please check your provider settings.';
  }
  
  if (/api key/i.test(message) || /unauthorized/i.test(message) || /authentication/i.test(message)) {
    return 'Invalid API key. Please check your API key in Settings.';
  }
  
  if (/error sending request/i.test(message)) {
    return 'Could not reach AI provider. Please check your settings and internet connection.';
  }
  
  // Generic provider error
  return 'AI provider error. Please check your settings.';
};

/**
 * Common error messages as constants to avoid duplication.
 */
export const ERROR_MESSAGES = {
  LOAD_HISTORY_FAILED: 'Failed to load history',
  LOAD_VOCAB_FAILED: 'Failed to load vocab notebook',
  UPDATE_VOCAB_FAILED: 'Failed to update vocab notebook',
  CLEAR_HISTORY_FAILED: 'Failed to clear history',
  DELETE_ITEM_FAILED: 'Failed to delete item',
  LOAD_SETTINGS_FAILED: 'Failed to load settings',
  UPDATE_SETTINGS_FAILED: 'Failed to update settings',
  UPDATE_PROVIDER_FAILED: 'Failed to update provider',
  LOAD_MODELS_FAILED: 'Failed to load Ollama models',
  SEARCH_FAILED: 'Failed to fetch definition',
  PHRASE_SEARCH_FAILED: 'Failed to fetch phrase definition',
  EXPLORATION_FAILED: 'Failed to fetch exploration data',
  GENERATE_EXAMPLES_FAILED: 'Failed to generate examples',
  GENERATE_EXERCISES_FAILED: 'Failed to generate more exercises',
  EXPORT_NOT_READY: 'Word data is not ready to export',
  EXPORT_MARKDOWN_FAILED: 'Failed to save markdown file',
  EXPORT_CAPACITIES_FAILED: 'Failed to share content with Capacities',
  EXPORT_CONFIG_MISSING: 'Add your Capacities token and space in Settings',
  PROVIDER_NOT_CONFIGURED: 'AI provider is not configured. Go to Settings to set up your API key.',
} as const;
