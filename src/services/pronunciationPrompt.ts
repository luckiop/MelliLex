export type EngineChoice = 'kokoro' | 'system';
export type PronunciationChoice = EngineChoice | 'cancelled';

export interface VoiceInstallStatus {
  pluginReady: boolean;
  runtimeReady: boolean;
  modelReady: boolean;
}

export const MISSING_VOICE_STATUS: VoiceInstallStatus = {
  pluginReady: true,
  runtimeReady: false,
  modelReady: false,
};

export type PronunciationPrompt =
  | { phase: 'choose'; status: VoiceInstallStatus }
  | { phase: 'downloading'; progress: number }
  | { phase: 'preparing' }
  | { phase: 'error'; message: string; status: VoiceInstallStatus };

type Listener = (prompt: PronunciationPrompt | null) => void;

const listeners = new Set<Listener>();
let prompt: PronunciationPrompt | null = null;
let resolver: ((choice: PronunciationChoice) => void) | null = null;

function emit() {
  for (const listener of listeners) listener(prompt);
}

export function subscribePronunciationPrompt(listener: Listener): () => void {
  listeners.add(listener);
  listener(prompt);
  return () => listeners.delete(listener);
}

export function requestPronunciationChoice(
  phase: 'choose' | 'error',
  message = '',
  status: VoiceInstallStatus = MISSING_VOICE_STATUS,
): Promise<PronunciationChoice> {
  return new Promise((resolve) => {
    resolver = resolve;
    prompt = phase === 'choose'
      ? { phase: 'choose', status }
      : { phase: 'error', message, status };
    emit();
  });
}

export function submitPronunciationChoice(choice: EngineChoice) {
  const resolve = resolver;
  resolver = null;
  resolve?.(choice);
}

export function reportPronunciationProgress(phase: 'downloading' | 'preparing', progress = 0) {
  prompt = phase === 'preparing' ? { phase: 'preparing' } : { phase: 'downloading', progress };
  emit();
}

export function cancelPronunciationPrompt() {
  const resolve = resolver;
  resolver = null;
  prompt = null;
  emit();
  resolve?.('cancelled');
}

export function clearPronunciationPrompt() {
  prompt = null;
  emit();
}
