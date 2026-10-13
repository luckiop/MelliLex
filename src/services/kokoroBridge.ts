import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { PronunciationBackend } from './pronunciationFlow';

interface DownloadProgress {
  progress: number;
}

let currentAudio: HTMLAudioElement | null = null;
let currentUrl: string | null = null;

function stopAudio() {
  if (currentAudio) {
    currentAudio.pause();
    currentAudio.src = '';
    currentAudio = null;
  }
  if (currentUrl) {
    URL.revokeObjectURL(currentUrl);
    currentUrl = null;
  }
}

function decodeWav(wavBase64: string): Uint8Array {
  const binary = atob(wavBase64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export function createKokoroBridge(): PronunciationBackend {
  return {
    isInstalled() {
      return invoke<boolean>('kokoro_status');
    },
    async download(onProgress) {
      const unlisten = await listen<DownloadProgress>('kokoro-download-progress', (event) => {
        onProgress(event.payload.progress);
      });
      try {
        await invoke('kokoro_download');
      } finally {
        unlisten();
      }
    },
    async synthesize(text) {
      const wavBase64 = await invoke<string>('kokoro_synthesize', { text });
      return decodeWav(wavBase64);
    },
    async playAudio(bytes) {
      stopAudio();
      const copy = new Uint8Array(bytes.byteLength);
      copy.set(bytes);
      const url = URL.createObjectURL(new Blob([copy], { type: 'audio/wav' }));
      currentUrl = url;
      const audio = new Audio(url);
      currentAudio = audio;
      audio.onended = () => {
        if (currentUrl === url) stopAudio();
      };
      await audio.play();
    },
    stopAudio,
    reset() {
      stopAudio();
      return invoke('kokoro_reset');
    },
  };
}
