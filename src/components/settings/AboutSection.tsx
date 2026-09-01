import { observer } from 'mobx-react-lite';
import { useEffect, useState } from 'react';
import { ExternalLink, RefreshCw } from 'lucide-react';
import { Button } from '../ui/button';
import { revealItemInDir, openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import { useUpdateChecker } from '../../hooks/useUpdateChecker';
import { UpdateDialog } from '../UpdateDialog';
import { useStores } from '../../stores/RootStore';
import { api } from '../../services/api';

export const AboutSection = observer(() => {
  const { t } = useTranslation();
  const { settingsStore } = useStores();
  const [logDir, setLogDir] = useState('');
  const {
    status,
    currentVersion,
    newVersion,
    downloadProgress,
    error,
    checkForUpdate,
    downloadAndInstall,
    restartApp,
    dismiss,
  } = useUpdateChecker();

  const checking = status === 'checking';
  const showDialog = status === 'available' || status === 'downloading' || status === 'ready' || status === 'error';
  const devLogEnabled = settingsStore.settings.devLogEnabled ?? false;

  useEffect(() => {
    void api.getLogDir().then(setLogDir).catch(() => undefined);
  }, []);

  const handleCheckUpdates = async () => {
    const result = await checkForUpdate();
    if (result === 'up-to-date') {
      toast.success(t('settings.about.upToDate'));
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between py-2 border-b border-border">
            <span className="text-muted-foreground">{t('settings.about.version')}</span>
            <span className="font-mono text-foreground">{currentVersion}</span>
          </div>
        </div>
      </div>

      <div>
        <Button
          onClick={handleCheckUpdates}
          disabled={checking}
          className="w-full"
          variant="outline"
        >
          <RefreshCw className={`w-4 h-4 mr-2 ${checking ? 'animate-spin' : ''}`} />
          {checking ? t('settings.about.checkingUpdates') : t('settings.about.checkUpdates')}
        </Button>
      </div>

      <div>
        <div className="space-y-2">
          <button
            onClick={() => openUrl('https://github.com/trrahul/MelliLex')}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left"
          >
            <span className="text-sm font-medium">{t('settings.about.github')}</span>
            <ExternalLink className="w-4 h-4 text-muted-foreground" />
          </button>

          <button
            onClick={() => openUrl('https://github.com/trrahul/MelliLex/issues')}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left"
          >
            <span className="text-sm font-medium">{t('settings.about.reportIssue')}</span>
            <ExternalLink className="w-4 h-4 text-muted-foreground" />
          </button>

          <button
            onClick={() => openUrl('https://github.com/trrahul/MelliLex/blob/master/LICENSE')}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left"
          >
            <span className="text-sm font-medium">{t('settings.about.license')}</span>
            <ExternalLink className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>
      </div>

      <div>
        <div className="flex items-center justify-between gap-4">
          <div className="space-y-0.5 pr-4">
            <label className="text-sm font-medium text-foreground">
              {t('settings.about.devLog')}
            </label>
            <p className="text-xs text-muted-foreground">{t('settings.about.devLogDesc')}</p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={devLogEnabled}
              onChange={(event) => {
                void settingsStore.updateSettings({ devLogEnabled: event.target.checked });
              }}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-muted peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
          </label>
        </div>
        {logDir ? (
          <p className="mt-2 text-xs text-muted-foreground break-all">
            {t('settings.about.devLogPath')}: {logDir}
          </p>
        ) : null}
        <Button
          type="button"
          variant="outline"
          className="w-full mt-3"
          disabled={!logDir}
          onClick={() => {
            if (logDir) {
              void revealItemInDir(logDir).catch(() => toast.error(t('vocab.updateFailed')));
            }
          }}
        >
          {t('settings.about.openLogs')}
        </Button>
      </div>

      <UpdateDialog
        open={showDialog}
        status={status}
        currentVersion={currentVersion}
        newVersion={newVersion}
        downloadProgress={downloadProgress}
        error={error}
        onDownload={downloadAndInstall}
        onRestart={restartApp}
        onDismiss={dismiss}
      />
    </div>
  );
});
