// 0.13.0 storage screen. Map: src/release/v0_13_0.ts.
// Lookup results, spoken audio, and logs each have an age delete. The voice
// model, speech libraries, and database are shown and are not cleared here.
import { observer } from 'mobx-react-lite';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../services/api';
import { toast } from 'sonner';
import type { StorageKind, StorageReport, StorageSlice } from '../../types';

const AGES = [7, 30, 90] as const;
type Age = (typeof AGES)[number];

const CLEARABLE: StorageKind[] = ['lookups', 'voice', 'logs'];

const SEGMENT_CLASS = [
  'bg-foreground/25',
  'bg-foreground/45',
  'bg-primary/45',
  'bg-primary/70',
  'bg-primary',
];

export const CacheSection = observer(() => {
  const { t } = useTranslation();
  const [report, setReport] = useState<StorageReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [ages, setAges] = useState<Record<StorageKind, Age>>({ lookups: 30, voice: 30, logs: 30 });
  const [pending, setPending] = useState<{ kind: StorageKind; all: boolean } | null>(null);
  const [clearing, setClearing] = useState(false);

  const load = async () => {
    setLoading(true);
    setError(false);
    try {
      setReport(await api.storageReport());
    } catch (err) {
      console.error('Failed to read storage:', err);
      setError(true);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const clear = async (kind: StorageKind, all: boolean) => {
    if (!pending || pending.kind !== kind || pending.all !== all) {
      setPending({ kind, all });
      return;
    }
    setClearing(true);
    try {
      const result = await api.storageClear(kind, all ? null : ages[kind]);
      setPending(null);
      toast.success(
        result.skipped > 0
          ? t('settings.cache.clearedSkipped', { n: result.removed, skipped: result.skipped })
          : t('settings.cache.cleared', { n: result.removed })
      );
      setReport(await api.storageReport());
    } catch (err) {
      console.error('Failed to clear storage:', err);
      toast.error(t('errors.cacheOperation'));
    } finally {
      setClearing(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-foreground mb-2">{t('settings.cache.heading')}</h3>
        <p className="text-sm text-muted-foreground">{t('settings.cache.intro')}</p>
      </div>

      {loading && <p className="text-sm text-muted-foreground">{t('settings.cache.loading')}</p>}
      {error && (
        <div className="space-y-2">
          <p className="text-sm text-destructive">{t('settings.cache.loadFailed')}</p>
          <button
            type="button"
            onClick={() => void load()}
            className="px-3 py-1.5 text-sm border border-border rounded-lg hover:bg-accent"
          >
            {t('settings.cache.refresh')}
          </button>
        </div>
      )}

      {report && (
        <>
          <div className="space-y-3">
            <p className="text-sm font-medium text-foreground tabular-nums">
              {t('settings.cache.total', { size: formatBytes(report.totalBytes) })}
            </p>
            <UsageBar report={report} labels={{
              database: t('settings.cache.database'),
              model: t('settings.cache.model'),
              runtime: t('settings.cache.runtime'),
              voice: t('settings.cache.voice'),
              logs: t('settings.cache.logs'),
            }} />
          </div>

          <section className="space-y-3">
            <h4 className="text-sm font-medium text-foreground">{t('settings.cache.clearable')}</h4>
            {CLEARABLE.map((kind) => (
              <ClearableRow
                key={kind}
                kind={kind}
                slice={report[kind]}
                age={ages[kind]}
                pending={pending?.kind === kind ? pending.all : null}
                clearing={clearing}
                onAge={(age) => {
                  setAges((current) => ({ ...current, [kind]: age }));
                  setPending(null);
                }}
                onClear={(all) => void clear(kind, all)}
                onCancel={() => setPending(null)}
              />
            ))}
          </section>

          <section className="space-y-2">
            <h4 className="text-sm font-medium text-foreground">{t('settings.cache.kept')}</h4>
            <KeptRow
              label={t('settings.cache.model')}
              hint={t('settings.cache.modelHint')}
              bytes={report.modelBytes}
            />
            <KeptRow
              label={t('settings.cache.runtime')}
              hint={t('settings.cache.runtimeHint')}
              bytes={report.runtimeBytes}
            />
            <KeptRow
              label={t('settings.cache.database')}
              hint={t('settings.cache.databaseHint')}
              bytes={report.databaseBytes}
            />
          </section>
        </>
      )}
    </div>
  );
});

function UsageBar({
  report,
  labels,
}: {
  report: StorageReport;
  labels: Record<'database' | 'model' | 'runtime' | 'voice' | 'logs', string>;
}) {
  const parts = [
    { key: 'database', bytes: report.databaseBytes, label: labels.database },
    { key: 'model', bytes: report.modelBytes, label: labels.model },
    { key: 'runtime', bytes: report.runtimeBytes, label: labels.runtime },
    { key: 'voice', bytes: report.voice.bytes, label: labels.voice },
    { key: 'logs', bytes: report.logs.bytes, label: labels.logs },
  ].filter((part) => part.bytes > 0);
  const total = parts.reduce((sum, part) => sum + part.bytes, 0);

  return (
    <div className="space-y-2">
      <div
        className="flex h-2 w-full overflow-hidden rounded-full bg-muted"
        role="img"
        aria-label={parts.map((part) => `${part.label} ${formatBytes(part.bytes)}`).join(', ')}
      >
        {total === 0 ? null : parts.map((part, index) => (
          <div
            key={part.key}
            className={SEGMENT_CLASS[index % SEGMENT_CLASS.length]}
            style={{ width: `${(part.bytes / total) * 100}%` }}
          />
        ))}
      </div>
      <div className="flex flex-wrap gap-x-4 gap-y-1">
        {parts.map((part, index) => (
          <span key={part.key} className="inline-flex items-center gap-1.5 text-sm text-muted-foreground">
            <span className={`h-2 w-2 rounded-full ${SEGMENT_CLASS[index % SEGMENT_CLASS.length]}`} />
            {part.label}
            <span className="tabular-nums text-foreground">{formatBytes(part.bytes)}</span>
          </span>
        ))}
      </div>
    </div>
  );
}

function ClearableRow({
  kind,
  slice,
  age,
  pending,
  clearing,
  onAge,
  onClear,
  onCancel,
}: {
  kind: StorageKind;
  slice: StorageSlice;
  age: Age;
  pending: boolean | null;
  clearing: boolean;
  onAge: (age: Age) => void;
  onClear: (all: boolean) => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const older = age === 7 ? slice.older7 : age === 30 ? slice.older30 : slice.older90;
  const selectId = `storage-age-${kind}`;
  return (
    <div className="rounded-lg border border-border p-3 space-y-3">
      <div className="flex items-baseline justify-between gap-4">
        <div>
          <div className="text-sm font-medium text-foreground">{t(`settings.cache.${kind}`)}</div>
          <p className="text-sm text-muted-foreground">{t(`settings.cache.${kind}Hint`)}</p>
        </div>
        <div className="text-sm tabular-nums text-foreground">{formatBytes(slice.bytes)}</div>
      </div>
      <p className="text-sm text-muted-foreground">
        {t('settings.cache.meta', { n: slice.count })}
        {slice.oldestUnix != null && ` · ${t('settings.cache.oldest', { days: ageDays(slice.oldestUnix) })}`}
        {` · ${older > 0
          ? t('settings.cache.olderAmount', { size: formatBytes(older), days: age })
          : t('settings.cache.noneOlder', { days: age })}`}
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <label htmlFor={selectId} className="text-sm text-muted-foreground">
          {t('settings.cache.ageLabel')}
        </label>
        <select
          id={selectId}
          value={age}
          onChange={(event) => onAge(Number(event.target.value) as Age)}
          className="h-9 rounded-lg border border-border bg-background px-2 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          {AGES.map((option) => (
            <option key={option} value={option}>
              {t('settings.cache.days', { n: option })}
            </option>
          ))}
        </select>
        <button
          type="button"
          onClick={() => onClear(false)}
          disabled={clearing || slice.count === 0}
          className="px-3 py-1.5 text-sm border border-border rounded-lg hover:bg-accent disabled:opacity-50"
        >
          {t('settings.cache.deleteOlder')}
        </button>
        <button
          type="button"
          onClick={() => onClear(true)}
          disabled={clearing || slice.count === 0}
          className="px-3 py-1.5 text-sm border border-destructive/50 text-destructive rounded-lg hover:bg-destructive/10 disabled:opacity-50"
        >
          {t('settings.cache.deleteAll')}
        </button>
      </div>
      {pending != null && (
        <div className="flex flex-wrap items-center gap-2">
          <p className="text-sm text-foreground">
            {pending
              ? t('settings.cache.confirmAll', { kind: t(`settings.cache.${kind}`) })
              : t('settings.cache.confirmOlder', { kind: t(`settings.cache.${kind}`), days: age })}
          </p>
          <button
            type="button"
            onClick={onCancel}
            className="px-3 py-1.5 text-sm border border-border rounded-lg hover:bg-accent"
          >
            {t('common.cancel')}
          </button>
        </div>
      )}
    </div>
  );
}

function KeptRow({ label, hint, bytes }: { label: string; hint: string; bytes: number }) {
  return (
    <div className="flex items-baseline justify-between gap-4 rounded-lg border border-border p-3">
      <div>
        <div className="text-sm font-medium text-foreground">{label}</div>
        <p className="text-sm text-muted-foreground">{hint}</p>
      </div>
      <div className="text-sm tabular-nums text-foreground">{formatBytes(bytes)}</div>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
  if (bytes < 1024) return `${Math.round(bytes)} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function ageDays(unix: number) {
  return Math.max(0, Math.floor((Date.now() / 1000 - unix) / 86_400));
}
