import { X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { usePageHint } from '../../hooks/usePageHint';
import type { PageHintId } from '../../services/PageHintService';

interface PageHintBannerProps {
  message: string;
  onDismiss: () => void;
}

export function PageHintBanner({ message, onDismiss }: PageHintBannerProps) {
  const { t } = useTranslation();

  return (
    <div
      className="mb-6 flex items-start gap-3 rounded-lg border border-border bg-muted/40 px-4 py-3"
      role="status"
    >
      <p className="min-w-0 flex-1 text-sm leading-relaxed text-muted-foreground">
        {message}
      </p>
      <button
        type="button"
        onClick={onDismiss}
        aria-label={t('common.close')}
        className="-mr-1 -mt-0.5 rounded-md p-1 text-muted-foreground/70 transition-colors hover:bg-accent hover:text-foreground"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

export function PageHint({ pageId }: { pageId: PageHintId }) {
  const { t } = useTranslation();
  const { visible, dismiss } = usePageHint(pageId);

  if (!visible) {
    return null;
  }

  return (
    <PageHintBanner
      message={t(`pageHints.${pageId}`)}
      onDismiss={dismiss}
    />
  );
}
