import { useEffect, useState } from 'react';
import { observer } from 'mobx-react-lite';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useStores } from '../stores/RootStore';
import { Alert, AlertDescription } from '../components/ui/alert';
import { Button } from '../components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../components/ui/dialog';
import { toast } from 'sonner';
import type { WordHistory } from '../types';
import { ScrollText, Search } from 'lucide-react';
import { LookupHeatmap } from '../components/history/LookupHeatmap';

interface GroupedHistory {
  title: string;
  items: WordHistory[];
}

type TimeGroupKey = 'today' | 'yesterday' | 'thisWeek' | 'thisMonth' | 'older';

const getTimeGroupKey = (timestamp: number): TimeGroupKey => {
  const diffDays = Math.floor((Date.now() - timestamp * 1000) / (1000 * 60 * 60 * 24));
  if (diffDays === 0) return 'today';
  if (diffDays === 1) return 'yesterday';
  if (diffDays < 7) return 'thisWeek';
  if (diffDays < 30) return 'thisMonth';
  return 'older';
};

const formatTime = (timestamp: number, locale: string): string => {
  const date = new Date(timestamp * 1000);
  const diffDays = Math.floor((Date.now() - date.getTime()) / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return date.toLocaleTimeString(locale, { hour: 'numeric', minute: '2-digit' });
  } else if (diffDays < 7) {
    return date.toLocaleDateString(locale, { weekday: 'short' });
  } else {
    return date.toLocaleDateString(locale, { month: 'short', day: 'numeric' });
  }
};

export const History = observer(() => {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const { historyStore, searchCoordinator, lastPageStore } = useStores();
  const [showClearDialog, setShowClearDialog] = useState(false);

  useEffect(() => {
    historyStore.loadHistory(2000);
  }, [historyStore]);

  const handleClearAll = async () => {
    try {
      await historyStore.clearHistory();
      setShowClearDialog(false);
      toast.success(t('history.successCleared'));
    } catch (error) {
      toast.error(t('history.failedClear'));
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await historyStore.deleteItem(id);
      toast.success(t('history.itemDeleted'));
    } catch (error) {
      toast.error(t('history.failedDelete'));
    }
  };

  const handleWordClick = async (word: string) => {
    navigate(lastPageStore.page);
    await searchCoordinator.search(word, { source: 'history-entry' });
  };

  const allHistoryItems = historyStore.items;
  const historyItems = historyStore.filteredItems;
  const hasActiveFilter = historyStore.searchQuery.trim().length > 0;

  const groupKeys: TimeGroupKey[] = ['today', 'yesterday', 'thisWeek', 'thisMonth', 'older'];
  const groupedItems: GroupedHistory[] = groupKeys.reduce<GroupedHistory[]>((acc, key) => {
    const items = historyItems.filter((item) => getTimeGroupKey(item.timestamp) === key);
    if (items.length > 0) {
      acc.push({ title: t(`history.sections.${key}`), items });
    }
    return acc;
  }, []);

  const totalWords = allHistoryItems.length;
  const uniqueWords = new Set(allHistoryItems.map((item) => item.word.toLowerCase())).size;
  const thisWeekItems = allHistoryItems.filter((item) => {
    const key = getTimeGroupKey(item.timestamp);
    return key === 'today' || key === 'yesterday' || key === 'thisWeek';
  }).length;
  const avgPerDay = totalWords > 0 ? (totalWords / 30).toFixed(1) : '0';

  return (
    <div className="mx-auto px-6 py-12 max-w-3xl">
      <div className="mb-8">
        <h1 className="text-4xl font-bold mb-2">{t('history.title')}</h1>
        <p className="text-muted-foreground">{t('history.description')}</p>
      </div>

      {historyStore.loadingState === 'loading' && (
        <div className="flex justify-center py-20">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
        </div>
      )}

      {historyStore.error && (
        <Alert variant="destructive" className="mb-6">
          <AlertDescription>{historyStore.error}</AlertDescription>
        </Alert>
      )}

      {historyStore.loadingState !== 'loading' && (
        <>
          {totalWords > 0 && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-2 mb-6">
              <div className="bg-card border border-border rounded-lg px-4 py-3.5 hover:border-foreground transition-colors">
                <div className="text-2xl font-semibold mb-0.5">{totalWords}</div>
                <div className="text-xs text-muted-foreground">{t('history.stats.totalWords')}</div>
              </div>
              <div className="bg-card border border-border rounded-lg px-4 py-3.5 hover:border-foreground transition-colors">
                <div className="text-2xl font-semibold mb-0.5">{uniqueWords}</div>
                <div className="text-xs text-muted-foreground">{t('history.stats.unique')}</div>
              </div>
              <div className="bg-card border border-border rounded-lg px-4 py-3.5 hover:border-foreground transition-colors">
                <div className="text-2xl font-semibold mb-0.5">{thisWeekItems}</div>
                <div className="text-xs text-muted-foreground">{t('history.stats.thisWeek')}</div>
              </div>
              <div className="bg-card border border-border rounded-lg px-4 py-3.5 hover:border-foreground transition-colors">
                <div className="text-2xl font-semibold mb-0.5">{avgPerDay}</div>
                <div className="text-xs text-muted-foreground">{t('history.stats.perDay')}</div>
              </div>
            </div>
          )}

          {totalWords > 0 && <LookupHeatmap items={allHistoryItems} />}

          {totalWords > 0 && (
            <div className="flex justify-end mb-4">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowClearDialog(true)}
              >
                {t('history.clear')}
              </Button>
            </div>
          )}

          {!hasActiveFilter && allHistoryItems.length === 0 && (
            <div className="text-center py-20 bg-card border border-border rounded-lg">
              <div className="flex justify-center mb-4 opacity-50">
                <ScrollText className="h-12 w-12" />
              </div>
              <div className="text-base font-medium mb-1">{t('history.empty')}</div>
              <div className="text-sm text-muted-foreground">
                {t('history.emptyDesc')}
              </div>
            </div>
          )}

          {hasActiveFilter && historyItems.length === 0 && (
            <div className="text-center py-20 bg-card border border-border rounded-lg">
              <div className="flex justify-center mb-4 opacity-50">
                <Search className="h-12 w-12" />
              </div>
              <div className="text-base font-medium mb-1">{t('history.noMatches')}</div>
              <div className="text-sm text-muted-foreground">
                {t('history.noMatchesDesc')}
              </div>
            </div>
          )}

          {groupedItems.map((group) => (
            <div key={group.title} className="mb-6">
              <div className="flex justify-between items-center mb-2 px-1">
                <div className="text-xs font-semibold text-muted-foreground tracking-wide">
                  {group.title}
                </div>
                <div className="text-xs text-muted-foreground">
                  {group.items.length} {group.items.length === 1 ? 'word' : 'words'}
                </div>
              </div>

              <div className="bg-card border border-border rounded-lg overflow-hidden">
                <div className="grid grid-cols-[1fr_120px_100px_40px] gap-3 items-center px-4 py-2.5 bg-muted border-b border-border">
                  <div className="text-xs font-medium text-muted-foreground">{t('history.table.word')}</div>
                  <div className="text-xs font-medium text-muted-foreground text-right">{t('history.table.time')}</div>
                  <div className="text-xs font-medium text-muted-foreground text-center">{t('history.table.provider')}</div>
                  <div></div>
                </div>

                {group.items.map((item) => (
                  <div
                    key={item.id}
                    onClick={() => handleWordClick(item.word)}
                    className="grid grid-cols-[1fr_120px_100px_40px] gap-3 items-center px-4 py-2.5 border-b border-border last:border-b-0 hover:bg-accent transition-colors cursor-pointer"
                  >
                    <div className="font-medium text-sm">{item.word}</div>
                    <div className="text-xs text-muted-foreground whitespace-nowrap text-right">
                      {formatTime(item.timestamp, i18n.language)}
                    </div>
                    <div className="flex justify-center">
                      <div className="bg-muted px-1.5 py-0.5 rounded text-[11px] font-medium text-muted-foreground">
                        {item.aiProvider}
                      </div>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(item.id);
                      }}
                      className="text-muted-foreground hover:text-foreground transition-colors px-1.5 py-0.5 text-sm"
                      title={t('history.deleteItem')}
                    >
                      ×
                    </button>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </>
      )}

      <Dialog open={showClearDialog} onOpenChange={setShowClearDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('history.clearConfirmTitle')}</DialogTitle>
            <DialogDescription>
              {t('history.clearConfirmDesc')}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowClearDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button variant="default" onClick={handleClearAll}>
              {t('history.clear')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
});
