import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '../ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import {
  submitPronunciationChoice,
  subscribePronunciationPrompt,
  type PronunciationPrompt,
  type VoiceInstallStatus,
} from '../../services/pronunciationPrompt';

function installLabel(status: VoiceInstallStatus, t: (key: string) => string) {
  if (!status.runtimeReady && !status.modelReady) return t('pronunciation.install');
  if (!status.runtimeReady) return t('pronunciation.installLibraries');
  return t('pronunciation.download');
}

export function PronunciationPromptDialog() {
  const { t } = useTranslation();
  const [prompt, setPrompt] = useState<PronunciationPrompt | null>(null);

  useEffect(() => subscribePronunciationPrompt(setPrompt), []);

  const busy = prompt?.phase === 'downloading' || prompt?.phase === 'preparing';
  const failed = prompt?.phase === 'error';
  const status = prompt?.phase === 'choose' || prompt?.phase === 'error' ? prompt.status : null;

  return (
    <Dialog
      open={prompt !== null}
      onOpenChange={(open) => {
        if (!open && !busy) submitPronunciationChoice('system');
      }}
    >
      <DialogContent
        className={busy ? 'sm:max-w-md [&>button.absolute]:hidden' : 'sm:max-w-md'}
        onPointerDownOutside={(event) => {
          if (busy) event.preventDefault();
        }}
        onEscapeKeyDown={(event) => {
          if (busy) event.preventDefault();
        }}
      >
        <DialogHeader>
          <DialogTitle>
            {failed ? t('pronunciation.errorTitle') : t('pronunciation.downloadTitle')}
          </DialogTitle>
          <DialogDescription>
            {prompt?.phase === 'downloading' && t('pronunciation.downloading')}
            {prompt?.phase === 'preparing' && t('pronunciation.preparing')}
            {prompt?.phase === 'choose' && t('pronunciation.downloadDescription')}
            {prompt?.phase === 'error' && (prompt.message || t('pronunciation.errorDescription'))}
          </DialogDescription>
        </DialogHeader>

        {status && (
          <ul className="space-y-2 text-sm">
            <li className="flex items-center justify-between gap-4">
              <span>{t('pronunciation.libraries')}</span>
              <span>{status.runtimeReady ? t('pronunciation.installed') : t('pronunciation.missing')}</span>
            </li>
            <li className="flex items-center justify-between gap-4">
              <span>{t('pronunciation.voiceModel')}</span>
              <span>{status.modelReady ? t('pronunciation.installed') : t('pronunciation.missing')}</span>
            </li>
          </ul>
        )}

        {status && !status.pluginReady && (
          <p className="text-sm text-muted-foreground">{t('pronunciation.pluginMissing')}</p>
        )}

        {prompt?.phase === 'downloading' && (
          <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-primary rounded-full transition-all duration-300 ease-out"
              style={{ width: `${prompt.progress}%` }}
            />
          </div>
        )}

        {!busy && status && (
          <DialogFooter>
            <Button variant="outline" onClick={() => submitPronunciationChoice('system')}>
              {t('pronunciation.useSystem')}
            </Button>
            <Button
              disabled={!status.pluginReady}
              onClick={() => submitPronunciationChoice('kokoro')}
            >
              {installLabel(status, t)}
            </Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
