import type { EngineChoice, PronunciationChoice } from './pronunciationPrompt';

const STORAGE_KEY = 'mellilex.pronunciationEngine';

export interface PronunciationBackend {
  isInstalled(): Promise<boolean>;
  download(onProgress: (progress: number) => void): Promise<void>;
  synthesize(text: string): Promise<Uint8Array>;
  playAudio(bytes: Uint8Array): Promise<void>;
  stopAudio(): void;
  reset(): Promise<void>;
}

export interface PronunciationFlowDeps {
  backend: PronunciationBackend;
  savedChoice(): EngineChoice | null;
  saveChoice(choice: EngineChoice): void;
  requestChoice(phase: 'choose' | 'error', message?: string): Promise<PronunciationChoice>;
  report(phase: 'downloading' | 'preparing', progress?: number): void;
  clearPrompt(): void;
  speakSystem(text: string): void;
  isCurrent(): boolean;
  takeText(): string;
}

export function readStoredEngine(): EngineChoice | null {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    if (value === 'kokoro' || value === 'system') return value;
  } catch {
    return null;
  }
  return null;
}

export function writeStoredEngine(choice: EngineChoice) {
  localStorage.setItem(STORAGE_KEY, choice);
}

export function clearStoredEngine() {
  localStorage.removeItem(STORAGE_KEY);
}

export async function speakWithPreferredEngine(deps: PronunciationFlowDeps): Promise<void> {
  if (!deps.isCurrent()) return;

  const saved = deps.savedChoice();
  if (saved === 'system') {
    speakIfCurrent(deps);
    return;
  }

  let replaceModel = false;
  if (saved === 'kokoro' || await deps.backend.isInstalled()) {
    try {
      if (await speakKokoro(deps, false)) return;
    } catch {
      replaceModel = true;
    }
  }

  if (!deps.isCurrent()) return;
  let phase: 'choose' | 'error' = 'choose';
  let message = '';
  for (;;) {
    const choice = await deps.requestChoice(phase, message);
    if (choice === 'cancelled' || !deps.isCurrent()) {
      deps.clearPrompt();
      return;
    }
    if (choice === 'system') {
      deps.saveChoice('system');
      deps.clearPrompt();
      speakIfCurrent(deps);
      return;
    }

    try {
      if (replaceModel) await deps.backend.reset();
      deps.report('downloading', 0);
      await deps.backend.download((progress) => deps.report('downloading', progress));
      if (!deps.isCurrent()) {
        deps.clearPrompt();
        return;
      }
      deps.saveChoice('kokoro');
      replaceModel = true;
      if (await speakKokoro(deps, true)) return;
      phase = 'error';
      message = '';
    } catch (error) {
      replaceModel = true;
      phase = 'error';
      message = error instanceof Error ? error.message : String(error);
    }
  }
}

async function speakKokoro(deps: PronunciationFlowDeps, announce: boolean): Promise<boolean> {
  if (!deps.isCurrent()) return false;
  const text = deps.takeText();
  if (!text) return true;
  if (announce) deps.report('preparing');
  const bytes = await deps.backend.synthesize(text);
  if (!deps.isCurrent()) return false;
  deps.clearPrompt();
  await deps.backend.playAudio(bytes);
  return true;
}

function speakIfCurrent(deps: PronunciationFlowDeps) {
  if (!deps.isCurrent()) return;
  const text = deps.takeText();
  if (!text) return;
  deps.speakSystem(text);
}
