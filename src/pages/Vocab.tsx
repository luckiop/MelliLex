import { useEffect } from 'react';
import { observer } from 'mobx-react-lite';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { BookMarked, Star } from 'lucide-react';
import { useStores } from '../stores/RootStore';
import { Alert, AlertDescription } from '../components/ui/alert';
import { toast } from 'sonner';
import type { VocabCategory } from '../types';

export const Vocab = observer(() => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { vocabStore, searchCoordinator, lastPageStore } = useStores();

  useEffect(() => {
    void vocabStore.loadList();
  }, [vocabStore, vocabStore.starFilter, vocabStore.categoryFilter]);

  const handleWordClick = async (word: string) => {
    navigate(lastPageStore.page);
    await searchCoordinator.search(word, { source: 'vocab-entry' });
  };

  const handleDelete = async (word: string) => {
    try {
      await vocabStore.deleteWord(word);
      toast.success(t('vocab.itemDeleted'));
    } catch {
      toast.error(t('vocab.failedDelete'));
    }
  };

  const starFilters: Array<{ id: number | 'all'; label: string }> = [
    { id: 'all', label: t('vocab.filterAll') },
    { id: 1, label: t('vocab.oneStar') },
    { id: 2, label: t('vocab.twoStars') },
  ];

  const categoryFilters: Array<{ id: VocabCategory | 'all'; label: string }> = [
    { id: 'all', label: t('vocab.filterAll') },
    { id: 'daily', label: t('vocab.daily') },
    { id: 'technical', label: t('vocab.technical') },
  ];

  return (
    <div className="mx-auto px-6 py-12 max-w-3xl">
      <div className="mb-8">
        <h1 className="text-4xl font-bold mb-2">{t('vocab.title')}</h1>
        <p className="text-muted-foreground">{t('vocab.description')}</p>
      </div>

      <div className="flex flex-wrap gap-2 mb-4">
        {starFilters.map((filter) => (
          <button
            key={`star-${filter.id}`}
            type="button"
            onClick={() => vocabStore.setStarFilter(filter.id)}
            className={`px-3 py-1.5 rounded-full border text-xs font-medium transition-colors ${
              vocabStore.starFilter === filter.id
                ? 'border-primary bg-primary text-primary-foreground'
                : 'border-border text-muted-foreground hover:text-foreground'
            }`}
          >
            {filter.label}
          </button>
        ))}
      </div>
      <div className="flex flex-wrap gap-2 mb-6">
        {categoryFilters.map((filter) => (
          <button
            key={`cat-${filter.id}`}
            type="button"
            onClick={() => vocabStore.setCategoryFilter(filter.id)}
            className={`px-3 py-1.5 rounded-full border text-xs font-medium transition-colors ${
              vocabStore.categoryFilter === filter.id
                ? 'border-primary bg-primary text-primary-foreground'
                : 'border-border text-muted-foreground hover:text-foreground'
            }`}
          >
            {filter.label}
          </button>
        ))}
      </div>

      {vocabStore.isLoading && (
        <div className="flex justify-center py-20">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
        </div>
      )}

      {vocabStore.error && (
        <Alert variant="destructive" className="mb-6">
          <AlertDescription>{vocabStore.error}</AlertDescription>
        </Alert>
      )}

      {!vocabStore.isLoading && vocabStore.items.length === 0 && (
        <div className="text-center py-20 bg-card border border-border rounded-lg">
          <div className="flex justify-center mb-4 opacity-50">
            <BookMarked className="h-12 w-12" />
          </div>
          <div className="text-base font-medium mb-1">{t('vocab.empty')}</div>
          <div className="text-sm text-muted-foreground">{t('vocab.emptyDesc')}</div>
        </div>
      )}

      {!vocabStore.isLoading && vocabStore.items.length > 0 && (
        <div className="bg-card border border-border rounded-lg overflow-hidden">
          {vocabStore.items.map((item) => (
            <div
              key={item.word}
              onClick={() => handleWordClick(item.word)}
              className="grid grid-cols-[1fr_auto_auto_40px] gap-3 items-center px-4 py-2.5 border-b border-border last:border-b-0 hover:bg-accent transition-colors cursor-pointer"
            >
              <div className="font-medium text-sm">{item.word}</div>
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <Star className="w-3.5 h-3.5 fill-current" />
                {item.stars === 1 ? t('vocab.oneStar') : t('vocab.twoStars')}
              </div>
              <div className="bg-muted px-1.5 py-0.5 rounded text-[11px] font-medium text-muted-foreground">
                {item.category === 'technical' ? t('vocab.technical') : t('vocab.daily')}
              </div>
              <button
                type="button"
                onClick={(event) => {
                  event.stopPropagation();
                  void handleDelete(item.word);
                }}
                className="text-muted-foreground hover:text-foreground transition-colors px-1.5 py-0.5 text-sm"
                title={t('vocab.deleteItem')}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
});
