import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '../ui/button';
import { useStores } from '../../stores/RootStore';
import { pronunciation } from '../../services/pronunciation';
import {
  VOICE_CHOICES,
  normalizeVoice,
  voiceIndex,
  type VoiceSettings,
} from '../../services/voicePrefs';

export function VoiceSection() {
  const { t } = useTranslation();
  const { settingsStore } = useStores();
  const stored = normalizeVoice(settingsStore.settings.voice);
  const [draft, setDraft] = useState<VoiceSettings>(stored);
  const [previewing, setPreviewing] = useState(false);
  const draftRef = useRef(draft);
  const saveTimer = useRef<number | null>(null);
  draftRef.current = draft;

  const storedKey = `${stored.volume}|${stored.speed}|${stored.speaker}`;
  const writing = useRef(false);

  useEffect(() => {
    if (saveTimer.current !== null || writing.current) return;
    setDraft(stored);
  }, [storedKey]);

  useEffect(() => () => {
    if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
  }, []);

  const persist = async (next: VoiceSettings) => {
    const voice = normalizeVoice(next);
    setDraft(voice);
    draftRef.current = voice;
    if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
    saveTimer.current = null;
    writing.current = true;
    try {
      await settingsStore.updateSettings({ voice });
    } finally {
      writing.current = false;
    }
  };

  const queue = (next: VoiceSettings) => {
    const voice = normalizeVoice(next);
    setDraft(voice);
    draftRef.current = voice;
    if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => {
      saveTimer.current = null;
      writing.current = true;
      void settingsStore.updateSettings({ voice: draftRef.current }).finally(() => {
        writing.current = false;
      });
    }, 180);
  };

  const selected = VOICE_CHOICES[voiceIndex(draft.speaker)];

  const preview = async () => {
    setPreviewing(true);
    try {
      await persist(draft);
      await pronunciation.play('hello');
    } finally {
      setPreviewing(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-foreground mb-2">{t('settings.voice.heading')}</h3>
        <p className="text-sm text-muted-foreground">{t('settings.voice.intro')}</p>
      </div>

      <VoiceSlider
        id="voice-volume"
        label={t('settings.voice.volume')}
        display={`${Math.round(draft.volume * 100)}%`}
        hint={t('settings.voice.volumeHint')}
        min={0.8}
        max={2.2}
        step={0.1}
        value={draft.volume}
        onChange={(volume) => queue({ ...draft, volume })}
      />

      <VoiceSlider
        id="voice-speed"
        label={t('settings.voice.speed')}
        display={`${draft.speed.toFixed(2)}×`}
        hint={t('settings.voice.speedHint')}
        min={0.7}
        max={1.3}
        step={0.05}
        value={draft.speed}
        onChange={(speed) => queue({ ...draft, speed: Number(speed.toFixed(2)) })}
      />

      <VoiceSlider
        id="voice-speaker"
        label={t('settings.voice.speaker')}
        display={`${t(`settings.voice.names.${selected.id}`)} · ${t(`settings.voice.effects.${selected.id}`)}`}
        hint={`${t('settings.voice.speakerHint')} ${t('settings.voice.speakerSystemNote')}`}
        min={0}
        max={VOICE_CHOICES.length - 1}
        step={1}
        value={voiceIndex(draft.speaker)}
        onChange={(index) => {
          const choice = VOICE_CHOICES[index];
          if (!choice) return;
          const next = { ...draft, speaker: choice.sid };
          queue(next);
        }}
      />

      <Button type="button" variant="outline" onClick={() => void preview()} disabled={previewing}>
        {previewing ? t('settings.voice.previewing') : t('settings.voice.preview')}
      </Button>
    </div>
  );
}

function VoiceSlider({
  id,
  label,
  display,
  hint,
  min,
  max,
  step,
  value,
  onChange,
}: {
  id: string;
  label: string;
  display: string;
  hint: string;
  min: number;
  max: number;
  step: number;
  value: number;
  onChange: (value: number) => void;
}) {
  const hintId = `${id}-hint`;
  return (
    <div className="space-y-2">
      <div className="flex items-baseline justify-between gap-4">
        <label htmlFor={id} className="text-sm font-medium text-foreground">
          {label}
        </label>
        <span className="text-sm text-foreground tabular-nums text-right">{display}</span>
      </div>
      <input
        id={id}
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        aria-describedby={hintId}
        onChange={(event) => onChange(Number(event.target.value))}
        className="h-2 w-full cursor-pointer accent-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
      />
      <p id={hintId} className="text-sm text-muted-foreground">
        {hint}
      </p>
    </div>
  );
}
