import {
  clearStoredEngine,
  readStoredEngine,
  speakWithPreferredEngine,
  writeStoredEngine,
  type PronunciationBackend,
} from './pronunciationFlow';
import {
  cancelPronunciationPrompt,
  clearPronunciationPrompt,
  reportPronunciationProgress,
  requestPronunciationChoice,
} from './pronunciationPrompt';

const ENGLISH_LANG = 'en-US';
const SPEAK_RATE = 0.9;
const REPLACE_DELAY_MS = 50;

const voiceBySynth = new WeakMap<SpeechSynthesis, SpeechSynthesisVoice | null>();
const watchedSynths = new WeakSet<SpeechSynthesis>();
let speakToken = 0;
let generation = 0;
let latestText = '';
let chain: Promise<void> = Promise.resolve();
let backendOverride: PronunciationBackend | null = null;
let overrideEnabled = false;
let productionBackend: PronunciationBackend | null = null;

function hasTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function usesKokoro(): boolean {
  return overrideEnabled || hasTauri();
}

async function resolveBackend(): Promise<PronunciationBackend | null> {
  if (overrideEnabled) return backendOverride;
  if (!hasTauri()) return null;
  if (!productionBackend) {
    const mod = await import('./kokoroBridge');
    productionBackend = mod.createKokoroBridge();
  }
  return productionBackend;
}

function stopAudible() {
  speakToken += 1;
  getSynthesis()?.cancel();
  (overrideEnabled ? backendOverride : productionBackend)?.stopAudio();
}

export function setPronunciationBackendForTests(backend: PronunciationBackend | null) {
  overrideEnabled = true;
  backendOverride = backend;
  latestText = '';
  generation += 1;
  chain = Promise.resolve();
}

export function resetPronunciationForTests() {
  overrideEnabled = false;
  backendOverride = null;
  productionBackend = null;
  latestText = '';
  generation += 1;
  chain = Promise.resolve();
  clearStoredEngine();
  clearPronunciationPrompt();
}

function getSynthesis(): SpeechSynthesis | null {
  if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
    return null;
  }
  return window.speechSynthesis ?? null;
}

function languageRank(lang: string): number {
  const normalized = lang.replace(/_/g, '-').toLowerCase();
  if (normalized === 'en-us') return 100;
  if (normalized.startsWith('en-us')) return 90;
  if (normalized === 'en-gb' || normalized === 'en-uk') return 60;
  if (normalized.startsWith('en')) return 40;
  return -1;
}

function selectEnglishVoice(voices: SpeechSynthesisVoice[]): SpeechSynthesisVoice | null {
  let best: SpeechSynthesisVoice | null = null;
  let bestScore = -1;

  for (const voice of voices) {
    const rank = languageRank(voice.lang);
    if (rank < 0) continue;

    let score = rank;
    if (/natural|neural/i.test(voice.name)) score += 30;
    if (voice.localService) score += 8;
    if (voice.default) score += 2;
    if (score > bestScore) {
      best = voice;
      bestScore = score;
    }
  }

  return best;
}

function resolveVoice(synth: SpeechSynthesis): SpeechSynthesisVoice | null {
  const voices = synth.getVoices();
  if (voices.length > 0) {
    const selected = selectEnglishVoice(voices);
    voiceBySynth.set(synth, selected);
    return selected;
  }
  return voiceBySynth.get(synth) ?? null;
}

function watchVoices(synth: SpeechSynthesis) {
  if (watchedSynths.has(synth)) return;
  watchedSynths.add(synth);
  synth.addEventListener('voiceschanged', () => {
    resolveVoice(synth);
  });
}

function speakNow(synth: SpeechSynthesis, text: string, token: number) {
  if (token !== speakToken) return;
  const utterance = new SpeechSynthesisUtterance(text);
  utterance.lang = ENGLISH_LANG;
  utterance.rate = SPEAK_RATE;
  const voice = resolveVoice(synth);
  if (voice) utterance.voice = voice;
  synth.speak(utterance);
}

function speakSystem(text: string) {
  const spoken = text.trim();
  if (!spoken) return;

  const synth = getSynthesis();
  if (!synth) return;

  watchVoices(synth);
  const token = ++speakToken;
  const interrupted = synth.speaking || synth.pending;
  if (interrupted) synth.cancel();

  if (interrupted) {
    window.setTimeout(() => speakNow(synth, spoken, token), REPLACE_DELAY_MS);
    return;
  }
  speakNow(synth, spoken, token);
}

async function drain() {
  const backend = await resolveBackend();
  const gen = generation;
  if (!backend) {
    const text = latestText;
    latestText = '';
    if (text && gen === generation) speakSystem(text);
    return;
  }

  while (latestText && gen === generation) {
    const captured = latestText;
    latestText = '';
    await speakWithPreferredEngine({
      backend,
      savedChoice: readStoredEngine,
      saveChoice: writeStoredEngine,
      requestChoice: requestPronunciationChoice,
      report: reportPronunciationProgress,
      clearPrompt: clearPronunciationPrompt,
      speakSystem,
      isCurrent: () => gen === generation,
      takeText: () => {
        if (latestText) {
          const next = latestText;
          latestText = '';
          return next;
        }
        return captured;
      },
    });
  }
}

export const pronunciation = {
  play(text: string): Promise<void> {
    const spoken = text.trim();
    if (!spoken) return Promise.resolve();
    if (!usesKokoro()) {
      speakSystem(spoken);
      return Promise.resolve();
    }

    latestText = spoken;
    stopAudible();
    const run = chain.then(() => drain());
    chain = run.then(() => undefined, () => undefined);
    return run;
  },

  stop(): void {
    generation += 1;
    latestText = '';
    stopAudible();
    cancelPronunciationPrompt();
  },
};
