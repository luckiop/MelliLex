import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { useStores } from '../../stores/RootStore';
import { Card } from '../ui/card';
import { ProviderErrorAlert } from '../ProviderErrorAlert';

export const ZhEnResultsView = observer(() => {
  const { t } = useTranslation();
  const { zhEnStore } = useStores();
  const result = zhEnStore.result;
  const query = result?.query || zhEnStore.currentQuery;

  const handleRetry = () => {
    const term = zhEnStore.currentQuery.trim();
    if (term) {
      void zhEnStore.search(term);
    }
  };

  if (zhEnStore.hasError) {
    return (
      <ProviderErrorAlert
        error={zhEnStore.error}
        onRetry={zhEnStore.currentQuery.trim() ? handleRetry : undefined}
      />
    );
  }

  if (zhEnStore.isLoading) {
    return (
      <div className="space-y-4">
        <div className="text-center mb-8">
          <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
            {t('zhEn.title')}
          </div>
          <h1 className="text-4xl font-bold tracking-tight">{query || '…'}</h1>
        </div>
        {[0, 1, 2].map((index) => (
          <Card key={index} className="p-5 animate-pulse">
            <div className="h-5 w-40 bg-muted rounded mb-3" />
            <div className="h-4 w-full bg-muted rounded mb-2" />
            <div className="h-4 w-2/3 bg-muted rounded" />
          </Card>
        ))}
      </div>
    );
  }

  if (!result || result.equivalents.length === 0) {
    return null;
  }

  return (
    <div className="space-y-4">
      <div className="text-center mb-8">
        <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
          {t('zhEn.title')}
        </div>
        <h1 className="text-4xl font-bold tracking-tight">{query}</h1>
        <p className="text-sm text-muted-foreground mt-2">{t('zhEn.subtitle')}</p>
      </div>

      {result.equivalents.map((item, index) => (
        <Card key={`${item.expression}-${index}`} className="p-5 define-card">
          <div className="flex flex-wrap items-baseline gap-2 mb-2">
            <span className="text-xl font-semibold">{item.expression}</span>
            {item.partOfSpeech ? (
              <span className="text-xs font-mono text-muted-foreground">{item.partOfSpeech}</span>
            ) : null}
          </div>
          {item.gloss ? (
            <p className="text-sm text-foreground mb-3">{item.gloss}</p>
          ) : null}
          {item.occasion ? (
            <div className="mb-3">
              <span className="inline-block text-xs font-medium px-2 py-0.5 rounded-full bg-muted text-muted-foreground">
                {item.occasion}
              </span>
            </div>
          ) : null}
          {item.example ? (
            <p className="text-sm italic text-muted-foreground mb-3">“{item.example}”</p>
          ) : null}
          {item.contrast ? (
            <p className="text-sm text-muted-foreground leading-relaxed">
              <span className="font-medium text-foreground">{t('zhEn.contrast')}: </span>
              {item.contrast}
            </p>
          ) : null}
        </Card>
      ))}
    </div>
  );
});
