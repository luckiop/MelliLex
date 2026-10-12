import { afterEach, describe, expect, it, vi } from 'vitest';
import { pronunciation } from '../pronunciation';

interface FakeVoice {
  name: string;
  lang: string;
  localService: boolean;
  default: boolean;
  voiceURI: string;
}

const zira: FakeVoice = {
  name: 'Microsoft Zira',
  lang: 'en-US',
  localService: true,
  default: false,
  voiceURI: 'zira',
};

const aria: FakeVoice = {
  name: 'Microsoft Aria Online (Natural)',
  lang: 'en-US',
  localService: false,
  default: false,
  voiceURI: 'aria',
};

const hazel: FakeVoice = {
  name: 'Microsoft Hazel',
  lang: 'en-GB',
  localService: true,
  default: false,
  voiceURI: 'hazel',
};

const huihui: FakeVoice = {
  name: 'Microsoft Huihui',
  lang: 'zh-CN',
  localService: true,
  default: true,
  voiceURI: 'huihui',
};

function installSynthesis(options?: { speaking?: boolean; voices?: FakeVoice[] }) {
  const speak = vi.fn();
  const cancel = vi.fn();
  const voices = options?.voices ?? [huihui, zira];
  vi.stubGlobal(
    'SpeechSynthesisUtterance',
    vi.fn(function Utterance(this: { text: string; lang: string; rate: number; voice: FakeVoice | null }, text: string) {
      this.text = text;
      this.lang = '';
      this.rate = 1;
      this.voice = null;
    })
  );
  Object.defineProperty(window, 'speechSynthesis', {
    configurable: true,
    writable: true,
    value: {
      speak,
      cancel,
      speaking: options?.speaking ?? false,
      pending: false,
      getVoices: () => voices,
      addEventListener: vi.fn(),
    },
  });
  return { speak, cancel };
}

function spokenUtterance(speak: ReturnType<typeof vi.fn>) {
  return speak.mock.calls[0][0] as { text: string; lang: string; rate: number; voice: FakeVoice | null };
}

describe('pronunciation', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.useRealTimers();
    Reflect.deleteProperty(window, 'speechSynthesis');
  });

  it('speaks trimmed English text with an en-US voice and a slower rate', () => {
    const { speak } = installSynthesis();

    pronunciation.play('  eloquent  ');

    expect(speak).toHaveBeenCalledOnce();
    expect(spokenUtterance(speak)).toMatchObject({
      text: 'eloquent',
      lang: 'en-US',
      rate: 0.9,
      voice: zira,
    });
  });

  it('prefers an English neural voice over a local en-US voice', () => {
    const { speak } = installSynthesis({ voices: [zira, aria, huihui] });

    pronunciation.play('eloquent');

    expect(spokenUtterance(speak).voice).toBe(aria);
  });

  it('falls back to any English voice when en-US is missing', () => {
    const { speak } = installSynthesis({ voices: [huihui, hazel] });

    pronunciation.play('eloquent');

    expect(spokenUtterance(speak).voice).toBe(hazel);
  });

  it('sets the language even when no English voice is installed', () => {
    const { speak } = installSynthesis({ voices: [huihui] });

    pronunciation.play('eloquent');

    expect(spokenUtterance(speak)).toMatchObject({ lang: 'en-US', voice: null });
  });

  it('uses voices that arrive after the first lookup', () => {
    const voices: FakeVoice[] = [];
    const listeners: Array<() => void> = [];
    const speak = vi.fn();
    vi.stubGlobal(
      'SpeechSynthesisUtterance',
      vi.fn(function Utterance(this: { text: string; lang: string; voice: FakeVoice | null }, text: string) {
        this.text = text;
        this.lang = '';
        this.voice = null;
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
        getVoices: () => voices,
        addEventListener: (_event: string, listener: () => void) => {
          listeners.push(listener);
        },
      },
    });

    pronunciation.play('eloquent');
    expect(spokenUtterance(speak).voice).toBeNull();

    voices.push(zira);
    listeners.forEach((listener) => listener());
    pronunciation.play('serendipity');

    expect(speak.mock.calls[1][0].voice).toBe(zira);
  });

  it('ignores blank text', () => {
    const { speak, cancel } = installSynthesis();

    pronunciation.play('   ');

    expect(speak).not.toHaveBeenCalled();
    expect(cancel).not.toHaveBeenCalled();
  });

  it('replaces speech that is already playing', async () => {
    vi.useFakeTimers();
    const { speak, cancel } = installSynthesis({ speaking: true });

    pronunciation.play('eloquent');

    expect(cancel).toHaveBeenCalledOnce();
    expect(speak).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(50);
    expect(spokenUtterance(speak).text).toBe('eloquent');
  });

  it('stops current and pending speech', async () => {
    vi.useFakeTimers();
    const { speak, cancel } = installSynthesis({ speaking: true });

    pronunciation.play('eloquent');
    pronunciation.stop();
    await vi.advanceTimersByTimeAsync(50);

    expect(cancel).toHaveBeenCalled();
    expect(speak).not.toHaveBeenCalled();
  });

  it('does nothing when speech synthesis is unavailable', () => {
    expect(() => pronunciation.play('eloquent')).not.toThrow();
    expect(() => pronunciation.stop()).not.toThrow();
  });
});
