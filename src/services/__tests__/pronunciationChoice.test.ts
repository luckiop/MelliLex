import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { pronunciation, resetPronunciationForTests, setPronunciationBackendForTests } from '../pronunciation';
import { readStoredEngine, writeStoredEngine, type PronunciationBackend } from '../pronunciationFlow';
import { submitPronunciationChoice, subscribePronunciationPrompt } from '../pronunciationPrompt';

function installSynthesis() {
  const speak = vi.fn();
  vi.stubGlobal(
    'SpeechSynthesisUtterance',
    vi.fn(function Utterance(this: { text: string }, text: string) {
      this.text = text;
    })
  );
  Object.defineProperty(window, 'speechSynthesis', {
    configurable: true,
    writable: true,
    value: {
      speak,
      cancel: vi.fn(),
      speaking: false,
      pending: false,
      getVoices: () => [],
      addEventListener: vi.fn(),
    },
  });
  return { speak };
}

function createBackend() {
  const backend: PronunciationBackend = {
    isInstalled: vi.fn().mockResolvedValue(false),
    download: vi.fn().mockImplementation(async (onProgress: (progress: number) => void) => {
      onProgress(100);
    }),
    synthesize: vi.fn().mockResolvedValue(new Uint8Array([1, 2, 3])),
    playAudio: vi.fn().mockResolvedValue(undefined),
    stopAudio: vi.fn(),
    reset: vi.fn().mockResolvedValue(undefined),
  };
  return backend;
}

async function choose(choice: 'kokoro' | 'system') {
  await vi.waitFor(() => {
    let pending = false;
    const unsubscribe = subscribePronunciationPrompt((prompt) => {
      pending = prompt?.phase === 'choose' || prompt?.phase === 'error';
    });
    unsubscribe();
    expect(pending).toBe(true);
  });
  submitPronunciationChoice(choice);
}

describe('pronunciation engine choice', () => {
  beforeEach(() => {
    resetPronunciationForTests();
  });

  afterEach(() => {
    resetPronunciationForTests();
    vi.unstubAllGlobals();
    Reflect.deleteProperty(window, 'speechSynthesis');
  });

  it('keeps the system voice when the download is declined', async () => {
    const { speak } = installSynthesis();
    const backend = createBackend();
    setPronunciationBackendForTests(backend);

    const playing = pronunciation.play('eloquent');
    await choose('system');
    await playing;

    expect(speak.mock.calls[0][0].text).toBe('eloquent');
    expect(backend.download).not.toHaveBeenCalled();
    expect(readStoredEngine()).toBe('system');

    speak.mockClear();
    await pronunciation.play('serendipity');
    expect(speak.mock.calls[0][0].text).toBe('serendipity');
    expect(backend.isInstalled).toHaveBeenCalledOnce();
  });

  it('downloads Kokoro once and then speaks with it', async () => {
    const backend = createBackend();
    setPronunciationBackendForTests(backend);

    const playing = pronunciation.play('  break the ice  ');
    await choose('kokoro');
    await playing;

    expect(backend.download).toHaveBeenCalledOnce();
    expect(backend.synthesize).toHaveBeenCalledWith('break the ice');
    expect(backend.playAudio).toHaveBeenCalledOnce();
    expect(readStoredEngine()).toBe('kokoro');

    vi.mocked(backend.isInstalled).mockResolvedValue(true);
    await pronunciation.play('eloquent');
    expect(backend.download).toHaveBeenCalledOnce();
    expect(backend.synthesize).toHaveBeenLastCalledWith('eloquent');
  });

  it('uses an installed Kokoro voice without asking again', async () => {
    const backend = createBackend();
    vi.mocked(backend.isInstalled).mockResolvedValue(true);
    setPronunciationBackendForTests(backend);

    await pronunciation.play('eloquent');

    expect(backend.download).not.toHaveBeenCalled();
    expect(backend.synthesize).toHaveBeenCalledWith('eloquent');
    expect(readStoredEngine()).toBeNull();
  });

  it('falls back to the system voice when the download fails', async () => {
    const { speak } = installSynthesis();
    const backend = createBackend();
    vi.mocked(backend.download).mockRejectedValue(new Error('offline'));
    setPronunciationBackendForTests(backend);

    const playing = pronunciation.play('eloquent');
    await choose('kokoro');
    await choose('system');
    await playing;

    expect(speak.mock.calls[0][0].text).toBe('eloquent');
    expect(readStoredEngine()).toBe('system');
  });

  it('does not ask when the system voice was already chosen', async () => {
    const { speak } = installSynthesis();
    const backend = createBackend();
    writeStoredEngine('system');
    setPronunciationBackendForTests(backend);

    await pronunciation.play('eloquent');

    expect(backend.isInstalled).not.toHaveBeenCalled();
    expect(speak.mock.calls[0][0].text).toBe('eloquent');
  });
});
