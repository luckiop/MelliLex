import type { VoiceSettings } from '../types';

export type { VoiceSettings };

export const DEFAULT_VOICE: VoiceSettings = {
  volume: 1.7,
  speed: 0.9,
  speaker: 3,
};

/** Ordered from a light, breathy voice toward a fuller, lower one. */
export const VOICE_CHOICES = [
  { sid: 4, id: 'sky' },
  { sid: 2, id: 'nicole' },
  { sid: 1, id: 'bella' },
  { sid: 8, id: 'isabella' },
  { sid: 0, id: 'af' },
  { sid: 3, id: 'sarah' },
  { sid: 7, id: 'emma' },
  { sid: 5, id: 'adam' },
  { sid: 6, id: 'michael' },
  { sid: 10, id: 'lewis' },
  { sid: 9, id: 'george' },
] as const;

export type VoiceId = (typeof VOICE_CHOICES)[number]['id'];

function clamp(value: number, min: number, max: number, fallback: number) {
  if (!Number.isFinite(value)) return fallback;
  return Math.min(max, Math.max(min, value));
}

export function normalizeVoice(voice?: Partial<VoiceSettings> | null): VoiceSettings {
  const speaker = voice?.speaker;
  const known = VOICE_CHOICES.some((choice) => choice.sid === speaker);
  return {
    volume: clamp(voice?.volume ?? DEFAULT_VOICE.volume, 0.8, 2.2, DEFAULT_VOICE.volume),
    speed: clamp(voice?.speed ?? DEFAULT_VOICE.speed, 0.7, 1.3, DEFAULT_VOICE.speed),
    speaker: known ? Number(speaker) : DEFAULT_VOICE.speaker,
  };
}

export function voiceIndex(speaker: number) {
  const index = VOICE_CHOICES.findIndex((choice) => choice.sid === speaker);
  return index >= 0 ? index : VOICE_CHOICES.findIndex((choice) => choice.sid === DEFAULT_VOICE.speaker);
}

let current = { ...DEFAULT_VOICE };

export function setVoicePrefs(voice?: Partial<VoiceSettings> | null) {
  current = normalizeVoice(voice);
}

export function getVoicePrefs(): VoiceSettings {
  return current;
}
