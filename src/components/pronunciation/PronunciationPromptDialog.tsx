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
} from '../../services/pronunciationPrompt';

export function PronunciationPromptDialog() {
  const { t } = useTranslation();
  const [prompt, setPrompt] = useState<PronunciationPrompt | null>(null);

  useEffect(() => subscribePronunciationPrompt(setPrompt), []);

  const busy = prompt?.phase === 'downloading' || prompt?.phase === 'preparing';
  const failed = prompt?.phase === 'error';

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

        {prompt?.phase === 'downloading' && (
          <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-primary rounded-full transition-all duration-300 ease-out"
              style={{ width: `${prompt.progress}%` }}
            />
          </div>
        )}

        {!busy && (
          <DialogFooter>
            <Button variant="outline" onClick={() => submitPronunciationChoice('system')}>
              {t('pronunciation.useSystem')}
            </Button>
            <Button onClick={() => submitPronunciationChoice('kokoro')}>
              {t('pronunciation.download')}
            </Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
