import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import type { WordHistory } from '../../types';
import {
  buildHeatmapDays,
  countsByDay,
  dayKey,
  heatmapLevel,
} from '../../utils/historyHeatmap';

const LEVEL_CLASS: Record<0 | 1 | 2 | 3 | 4, string> = {
  0: 'bg-muted',
  1: 'bg-primary/25',
  2: 'bg-primary/45',
  3: 'bg-primary/70',
  4: 'bg-primary',
};

interface Props {
  items: WordHistory[];
}

export function LookupHeatmap({ items }: Props) {
  const { t, i18n } = useTranslation();
  const days = useMemo(() => buildHeatmapDays(new Date()), []);
  const counts = useMemo(() => countsByDay(items), [items]);
  const max = useMemo(() => Math.max(0, ...counts.values()), [counts]);
  const quote = useMemo(() => {
    const lines = t('history.encouragement', { returnObjects: true });
    const list = Array.isArray(lines) ? lines.filter((line): line is string => typeof line === 'string') : [];
    if (list.length === 0) {
      return '';
    }
    return list[Math.floor(Math.random() * list.length)];
  }, [t, i18n.language]);

  const columns: Date[][] = [];
  for (let i = 0; i < days.length; i += 7) {
    columns.push(days.slice(i, i + 7));
  }
  if (columns.length > 0) {
    const last = columns[columns.length - 1];
    while (last.length < 7) {
      last.push(new Date(NaN));
    }
  }

  return (
    <div className="mb-6 rounded-lg border border-border bg-card px-4 py-4">
      <div className="mb-3 flex items-baseline justify-between gap-4">
        <h2 className="text-sm font-medium text-foreground">{t('history.heatmapTitle')}</h2>
        {quote && (
          <p className="text-xs text-muted-foreground text-right leading-relaxed">{quote}</p>
        )}
      </div>
      <div className="overflow-x-auto pb-1">
        <div className="flex gap-[3px] min-w-max">
          {columns.map((week, weekIndex) => (
            <div key={weekIndex} className="flex flex-col gap-[3px]">
              {week.map((day, dayIndex) => {
                if (Number.isNaN(day.getTime())) {
                  return <div key={`${weekIndex}-${dayIndex}`} className="h-[11px] w-[11px]" />;
                }
                const key = dayKey(day);
                const count = counts.get(key) ?? 0;
                const level = heatmapLevel(count, max);
                const label = t('history.heatmapCount', {
                  count,
                  date: day.toLocaleDateString(i18n.language, {
                    month: 'short',
                    day: 'numeric',
                    year: 'numeric',
                  }),
                });
                return (
                  <div
                    key={key}
                    title={label}
                    aria-label={label}
                    className={`h-[11px] w-[11px] rounded-[2px] ${LEVEL_CLASS[level]}`}
                  />
                );
              })}
            </div>
          ))}
        </div>
      </div>
      <div className="mt-3 flex items-center justify-end gap-1 text-[10px] text-muted-foreground">
        <span>{t('history.heatmapLess')}</span>
        {([0, 1, 2, 3, 4] as const).map((level) => (
          <span key={level} className={`h-[11px] w-[11px] rounded-[2px] ${LEVEL_CLASS[level]}`} />
        ))}
        <span>{t('history.heatmapMore')}</span>
      </div>
    </div>
  );
}
