import { observer } from 'mobx-react-lite';
import { Trans, useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { api } from '../../services/api';

interface GlobalLookupSectionProps {
  enabled: boolean;
  onEnabledChange: (enabled: boolean) => void;
  clipboardSearchEnabled: boolean;
  onClipboardSearchChange: (enabled: boolean) => void;
}

export const GlobalLookupSection = observer(({
  enabled,
  onEnabledChange,
  clipboardSearchEnabled,
  onClipboardSearchChange,
}: GlobalLookupSectionProps) => {
  const { t } = useTranslation();

  const handleEnabledToggle = async (newEnabled: boolean) => {
    try {
      if (newEnabled) {
        await api.enableGlobalLookup();
        toast.success(t('settings.globalLookup.enabled'));
      } else {
        await api.disableGlobalLookup();
        toast.success(t('settings.globalLookup.disabled'));
      }
      onEnabledChange(newEnabled);
    } catch (error) {
      console.error('Failed to toggle global lookup:', error);
      toast.error(t('settings.globalLookup.toggleFailed'));
    }
  };

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <label className="text-sm font-medium text-foreground">
              {t('settings.globalLookup.enable')}
            </label>
            <p className="text-xs text-muted-foreground">
              {t('settings.globalLookup.enableDesc')}
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => handleEnabledToggle(e.target.checked)}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-muted peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
          </label>
        </div>

        <div className="flex items-center justify-between border-t border-border pt-4">
          <div className="space-y-0.5">
            <label className="text-sm font-medium text-foreground">
              {t('settings.clipboardSearch.enable')}
            </label>
            <p className="text-xs text-muted-foreground">
              {t('settings.clipboardSearch.enableDesc')}
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={clipboardSearchEnabled}
              onChange={(e) => onClipboardSearchChange(e.target.checked)}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-muted peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-border after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
          </label>
        </div>
      </div>

      <div className="rounded-md border border-border bg-muted/70 px-4 py-3">
        <p className="mb-2 text-sm font-medium text-foreground">
          {t('settings.globalLookup.howToUse')}
        </p>
        <div className="space-y-2 text-sm leading-relaxed text-muted-foreground">
          <p className="font-medium text-foreground">{t('settings.globalLookup.instruction')}</p>
          <ul className="list-disc list-inside space-y-1 ml-1">
            <li>
              <Trans
                i18nKey="settings.globalLookup.methodMouse"
                components={{
                  1: <code className="rounded bg-background px-1.5 py-0.5 font-mono text-xs text-foreground" />,
                }}
              />
            </li>
          </ul>
          <p className="text-xs">{t('settings.globalLookup.note')}</p>
        </div>
      </div>
    </div>
  );
});
