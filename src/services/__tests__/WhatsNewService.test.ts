import { describe, it, expect, beforeEach } from 'vitest';
import { WhatsNewService } from '../WhatsNewService';

describe('WhatsNewService', () => {
  beforeEach(() => {
    WhatsNewService.resetForTests();
  });

  it('shows notes when no version has been seen yet', () => {
    expect(WhatsNewService.shouldShow('0.12.0')).toBe(true);
  });

  it('hides notes after the current version is marked seen', () => {
    WhatsNewService.markSeen('0.12.0');
    expect(WhatsNewService.shouldShow('0.12.0')).toBe(false);
  });

  it('shows notes again after a version bump', () => {
    WhatsNewService.markSeen('0.11.0');
    expect(WhatsNewService.shouldShow('0.12.0')).toBe(true);
  });
});
