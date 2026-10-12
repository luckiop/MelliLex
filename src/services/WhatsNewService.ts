const STORAGE_KEY = 'mellilex.lastSeenVersion';

export class WhatsNewService {
  static getLastSeenVersion(): string | null {
    try {
      const value = localStorage.getItem(STORAGE_KEY);
      return value && value.trim() ? value.trim() : null;
    } catch {
      return null;
    }
  }

  static shouldShow(currentVersion: string): boolean {
    const current = currentVersion.trim();
    if (!current) {
      return false;
    }
    return this.getLastSeenVersion() !== current;
  }

  static markSeen(currentVersion: string): void {
    const current = currentVersion.trim();
    if (!current) {
      return;
    }
    try {
      localStorage.setItem(STORAGE_KEY, current);
    } catch {
      // ignore quota / private-mode failures
    }
  }

  static resetForTests(): void {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {
      // ignore
    }
  }
}
