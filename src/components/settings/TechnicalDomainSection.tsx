import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Input } from '../ui/input';

export const DEFAULT_TECHNICAL_DOMAIN =
  'computer science, control engineering, and robotics';

const PRESETS = [
  { id: 'computing', value: DEFAULT_TECHNICAL_DOMAIN },
  { id: 'biology', value: 'biology' },
  { id: 'medicine', value: 'medicine' },
  { id: 'law', value: 'law' },
  { id: 'economics', value: 'economics' },
] as const;

interface TechnicalDomainSectionProps {
  value: string;
  onChange: (value: string) => void;
}

export function TechnicalDomainSection({ value, onChange }: TechnicalDomainSectionProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  const commit = (next: string) => {
    const trimmed = next.trim() || DEFAULT_TECHNICAL_DOMAIN;
    setDraft(trimmed);
    if (trimmed !== value) {
      onChange(trimmed);
    }
  };

  return (
    <div className="space-y-3 border-t border-border pt-6">
      <div className="space-y-0.5">
        <label htmlFor="technical-domain" className="text-sm font-medium text-foreground">
          {t('settings.technicalDomain.title')}
        </label>
        <p className="text-xs text-muted-foreground">
          {t('settings.technicalDomain.desc')}
        </p>
      </div>
      <Input
        id="technical-domain"
        value={draft}
        placeholder={t('settings.technicalDomain.placeholder')}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={() => commit(draft)}
        onKeyDown={(event) => {
          if (event.key === 'Enter') {
            event.preventDefault();
            commit(draft);
          }
        }}
      />
      <div className="flex flex-wrap gap-2">
        {PRESETS.map((preset) => {
          const selected = value === preset.value;
          return (
            <button
              key={preset.id}
              type="button"
              onClick={() => commit(preset.value)}
              className={`rounded-full px-3 py-1 text-xs transition-colors ${
                selected
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-secondary text-secondary-foreground hover:bg-accent'
              }`}
            >
              {t(`settings.technicalDomain.presets.${preset.id}`)}
            </button>
          );
        })}
      </div>
    </div>
  );
}
