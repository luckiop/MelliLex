import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { useStores } from '../../stores/RootStore';
import { Card } from '../ui/card';
import { ProviderErrorAlert } from '../ProviderErrorAlert';

export const TranslateResultsView = observer(() => {
  const { t } = useTranslation();
  const { translateStore } = useStores();
  const result = translateStore.result;
  const source = result?.source || translateStore.currentSource;
  const remark = result?.remark?.trim();

  const handleRetry = () => {
    const term = translateStore.currentSource.trim();
    if (term) {
      void translateStore.search(term);
    }
  };

  if (translateStore.hasError) {
    return (
      <ProviderErrorAlert
        error={translateStore.error}
        onRetry={translateStore.currentSource.trim() ? handleRetry : undefined}
      />
    );
  }

  if (translateStore.isLoading) {
    return (
      <div className="space-y-4">
        <div className="text-center mb-8">
          <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
            {t('translate.title')}
          </div>
        </div>
        <Card className="p-5 animate-pulse">
          <div className="h-4 w-24 bg-muted rounded mb-3" />
          <div className="h-4 w-full bg-muted rounded mb-2" />
          <div className="h-4 w-5/6 bg-muted rounded" />
        </Card>
        <Card className="p-5 animate-pulse">
          <div className="h-4 w-16 bg-muted rounded mb-3" />
          <div className="h-4 w-full bg-muted rounded mb-2" />
          <div className="h-4 w-2/3 bg-muted rounded" />
        </Card>
      </div>
    );
  }

  if (!result || !result.translation.trim()) {
    return null;
  }

  return (
    <div className="space-y-4">
      <div className="text-center mb-6">
        <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
          {t('translate.title')}
        </div>
      </div>

      <Card className="p-5 define-card">
        <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
          {t('translate.source')}
        </div>
        <p className="text-[0.9375rem] leading-relaxed whitespace-pre-wrap">{source}</p>
      </Card>

      <Card className="p-5 define-card define-card--accent-amber">
        <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
          {t('translate.translation')}
        </div>
        <p className="text-[0.9375rem] leading-relaxed whitespace-pre-wrap">{result.translation}</p>
      </Card>

      {remark ? (
        <Card className="p-5 define-card define-card--accent-blue">
          <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
            {t('translate.remark')}
          </div>
          <p className="text-sm leading-relaxed text-foreground/90 whitespace-pre-wrap">{remark}</p>
        </Card>
      ) : null}
    </div>
  );
});
