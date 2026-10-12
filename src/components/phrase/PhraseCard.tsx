import { observer } from 'mobx-react-lite';
import { useStores } from '../../stores/RootStore';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { RefreshCw } from 'lucide-react';
import { PhraseOverviewSection } from './PhraseOverviewSection';
import { PhraseContextSection } from './PhraseContextSection';
import { PhraseRelatedSection } from './PhraseRelatedSection';
import { PhraseOverviewSkeleton } from './PhraseOverviewSkeleton';
import { PhraseContextSkeleton } from './PhraseContextSkeleton';
import { PhraseRelatedSkeleton } from './PhraseRelatedSkeleton';

export const PhraseCard = observer(() => {
  const { phraseStore } = useStores();
  const {
    currentPhrase,
    overviewSection,
    contextSection,
    relatedSection,
    hasOverviewSection,
    hasContextSection,
    isLoading,
    error,
  } = phraseStore;

  const hasRelatedContent = Boolean(
    relatedSection &&
      (relatedSection.variations.length > 0 ||
        relatedSection.similarPhrases.length > 0 ||
        relatedSection.oppositePhrases.length > 0 ||
        relatedSection.seeAlso.length > 0)
  );

  if (error) {
    return (
      <Card className="p-6">
        <div className="text-center space-y-4">
          <p className="text-destructive">{error}</p>
          <Button
            variant="outline"
            onClick={() => phraseStore.searchPhrase(currentPhrase)}
          >
            <RefreshCw className="w-4 h-4 mr-2" />
            Try Again
          </Button>
        </div>
      </Card>
    );
  }

  if (!currentPhrase && !isLoading) {
    return null;
  }

  return (
    <div className="space-y-4">
      <Card className="p-6 define-card define-card--accent-blue">
        {hasOverviewSection && overviewSection ? (
          <PhraseOverviewSection data={overviewSection} />
        ) : (
          <PhraseOverviewSkeleton />
        )}
      </Card>

      {hasOverviewSection && (
        <Card className="p-6 animate-fadeIn define-card define-card--accent-amber">
          {hasContextSection && contextSection ? (
            <PhraseContextSection data={contextSection} phrase={currentPhrase} />
          ) : (
            <PhraseContextSkeleton />
          )}
        </Card>
      )}

      {hasOverviewSection && isLoading && !hasRelatedContent && (
        <Card className="p-6 animate-fadeIn define-card define-card--accent-plum">
          <PhraseRelatedSkeleton />
        </Card>
      )}

      {hasOverviewSection && hasRelatedContent && (
        <Card className="p-6 animate-fadeIn define-card define-card--accent-plum">
          <PhraseRelatedSection data={relatedSection!} />
        </Card>
      )}
    </div>
  );
});
