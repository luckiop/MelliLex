import { observer } from 'mobx-react-lite';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { save } from '@tauri-apps/plugin-dialog';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { useStores } from '../../stores/RootStore';

export const VocabSection = observer(() => {
  const { t } = useTranslation();
  const { settingsStore } = useStores();
  const enabled = settingsStore.settings.vocabNoteEnabled ?? false;
  const storedPath = settingsStore.settings.vocabNotePath ?? '';
  const [path, setPath] = useState(storedPath);

  useEffect(() => {
    setPath(storedPath);
  }, [storedPath]);

  const handleEnabledToggle = async (next: boolean) => {
    try {
      await settingsStore.updateSettings({ vocabNoteEnabled: next });
      if (settingsStore.error) {
        toast.error(settingsStore.error);
      }
    } catch {
      toast.error(t('vocab.settingsSaveFailed'));
    }
  };

  const handlePathChange = async (next: string) => {
    try {
      await settingsStore.updateSettings({ vocabNotePath: next });
      if (settingsStore.error) {
        toast.error(settingsStore.error);
        setPath(storedPath);
      }
    } catch {
      toast.error(t('vocab.settingsSaveFailed'));
      setPath(storedPath);
    }
  };

  const handleBrowse = async () => {
    const selected = await save({
      defaultPath: path || 'vocab.md',
      filters: [{ name: 'Markdown', extensions: ['md'] }],
    });
    if (typeof selected === 'string' && selected.length > 0) {
      await handlePathChange(selected);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="space-y-0.5 pr-4">
          <label className="text-sm font-medium text-foreground">
            {t('vocab.settingsEnable')}
          </label>
          <p className="text-xs text-muted-foreground">{t('vocab.settingsEnableDesc')}</p>
        </div>
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(event) => void handleEnabledToggle(event.target.checked)}
            className="sr-only peer"
          />
          <div className="w-11 h-6 bg-muted peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
        </label>
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-foreground">{t('vocab.settingsPath')}</label>
        <div className="flex gap-2">
          <Input
            value={path}
            disabled={!enabled}
            placeholder={t('vocab.settingsPathPlaceholder')}
            onChange={(event) => setPath(event.target.value)}
            onBlur={(event) => void handlePathChange(event.target.value)}
          />
          <Button type="button" variant="outline" disabled={!enabled} onClick={() => void handleBrowse()}>
            {t('vocab.browse')}
          </Button>
        </div>
        <p className="text-xs text-muted-foreground">{t('vocab.settingsPathHint')}</p>
      </div>

      <div className="flex items-center justify-between">
        <div className="space-y-0.5 pr-4">
          <label className="text-sm font-medium text-foreground">
            {t('vocab.settingsSplit')}
          </label>
          <p className="text-xs text-muted-foreground">{t('vocab.settingsSplitDesc')}</p>
        </div>
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            checked={settingsStore.settings.vocabNoteSplitFiles ?? false}
            disabled={!enabled}
            onChange={(event) => {
              void settingsStore.updateSettings({ vocabNoteSplitFiles: event.target.checked });
            }}
            className="sr-only peer"
          />
          <div className="w-11 h-6 bg-muted peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary peer-disabled:opacity-50"></div>
        </label>
      </div>
    </div>
  );
});
