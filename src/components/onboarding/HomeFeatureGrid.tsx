import { useState } from 'react';
import { useTranslation } from 'react-i18next';

const FEATURES = ['define', 'vocab', 'notes', 'technical', 'history', 'settings', 'api'] as const;
const DEEPSEEK_API_KEYS_URL = 'https://platform.deepseek.com/api_keys';

type FeatureId = (typeof FEATURES)[number];

function featureSteps(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((step): step is string => typeof step === 'string');
}

export function HomeFeatureGrid() {
  const { t } = useTranslation();
  const [openId, setOpenId] = useState<FeatureId | null>(null);
  const steps = openId ? featureSteps(t(`home.features.${openId}.steps`, { returnObjects: true })) : [];

  const openPlatform = () => {
    void import('@tauri-apps/plugin-opener')
      .then(({ openUrl }) => openUrl(DEEPSEEK_API_KEYS_URL))
      .catch(() => undefined);
  };

  return (
    <div className="mt-8" onMouseLeave={() => setOpenId(null)}>
      <ul className="flex flex-wrap justify-center gap-x-5 gap-y-1 text-sm">
        {FEATURES.map((id) => {
          const open = openId === id;
          return (
            <li key={id}>
              <button
                type="button"
                id={`home-feature-${id}`}
                aria-expanded={open}
                aria-controls="home-feature-panel"
                className={`text-muted-foreground transition-colors hover:text-foreground ${
                  open ? 'text-foreground underline decoration-primary underline-offset-4' : ''
                }`}
                onMouseEnter={() => setOpenId(id)}
                onFocus={() => setOpenId(id)}
              >
                {t(`home.features.${id}.title`)}
              </button>
            </li>
          );
        })}
      </ul>
      {openId && steps.length > 0 ? (
        <div
          id="home-feature-panel"
          role="region"
          aria-labelledby={`home-feature-${openId}`}
          className="mt-4 rounded-lg border border-border bg-card px-4 py-3 text-left text-sm text-foreground"
        >
          {openId === 'api' ? (
            <button
              type="button"
              className="mb-3 text-primary underline decoration-primary/50 underline-offset-4"
              onClick={openPlatform}
            >
              {t('home.features.api.open')}
            </button>
          ) : null}
          <ol className="list-decimal space-y-2 pl-5 leading-relaxed">
            {steps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
        </div>
      ) : null}
    </div>
  );
}
