import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { useEffect } from 'react';
import { useStores } from '../stores/RootStore';
import { useTypingAnimation } from '../hooks/useTypingAnimation';
import { Card } from '../components/ui/card';
import { ProviderErrorAlert } from '../components/ProviderErrorAlert';
import {
  Section1HeaderSkeleton,
  Section2MeaningsSkeleton,
  WordSection3RelatedSkeleton,
  Section1HeaderView,
  Section2MeaningsView,
  WordSection3RelatedView,
} from '../components/sections';
import { PhraseCard } from '../components/phrase';
import { ZhEnResultsView } from '../components/zhen/ZhEnResultsView';
import { HomeFeatureGrid } from '../components/onboarding/HomeFeatureGrid';
import { PageHint } from '../components/onboarding/PageHintBanner';

export const Home = observer(() => {
  const { t } = useTranslation();
  const { progressiveWordStore, phraseStore, zhEnStore, searchCoordinator, lastPageStore } = useStores();
  
  useEffect(() => {
    lastPageStore.setLastPage('/');
  }, [lastPageStore]);

  const { displayedWord, isTyping } = useTypingAnimation(
    progressiveWordStore.headerSection?.word
  );

  const hasWordData =
    progressiveWordStore.headerSection ||
    progressiveWordStore.meaningsSection ||
    progressiveWordStore.relatedSection;
    
  const hasPhraseData =
    phraseStore.overviewSection ||
    phraseStore.contextSection ||
    phraseStore.relatedSection;

  const hasZhEnData = Boolean(zhEnStore.result) || zhEnStore.isLoading || zhEnStore.hasError;
  const hasAnyData = hasWordData || hasPhraseData || hasZhEnData;
  const isWordLoading = progressiveWordStore.isLoading;
  const isPhraseLoading = phraseStore.isLoading;
  const isLoading = isWordLoading || isPhraseLoading || zhEnStore.isLoading;
  
  const showZhEnUI =
    hasZhEnData ||
    (zhEnStore.isLoading && searchCoordinator.currentInputType === 'zh-en');
  const showPhraseUI =
    !showZhEnUI &&
    (hasPhraseData || (isPhraseLoading && searchCoordinator.currentInputType === 'phrase'));

  const handleWordRetry = () => {
    const word = progressiveWordStore.currentWord?.trim();
    if (word) {
      progressiveWordStore.searchWord(word);
    }
  };
  
  const handlePhraseRetry = () => {
    const phrase = phraseStore.currentPhrase?.trim();
    if (phrase) {
      phraseStore.searchPhrase(phrase);
    }
  };

  return (
    <div className="container mx-auto px-8 py-12 max-w-3xl">
      <PageHint pageId="home" />
      {!hasAnyData && !isLoading && !progressiveWordStore.hasError && !phraseStore.hasError && !zhEnStore.hasError && (
        <div className="max-w-xl mx-auto text-center py-8">
          <h1 className="text-7xl font-bold text-foreground mb-6 tracking-tight">
            {t('home.title')}
          </h1>
          <p className="text-xl text-muted-foreground mb-8">
            {t('home.subtitle')}
          </p>
          <div className="space-y-2 text-sm text-muted-foreground">
            <p>{t('home.prompt')}</p>
            <p className="text-xs text-muted-foreground/60">
              {t('home.tryWords')}
            </p>
          </div>
          <HomeFeatureGrid />
        </div>
      )}

      {progressiveWordStore.hasError && (
        <ProviderErrorAlert 
          error={progressiveWordStore.error}
          onRetry={progressiveWordStore.currentWord?.trim() ? handleWordRetry : undefined}
        />
      )}

      {phraseStore.hasError && (
        <ProviderErrorAlert 
          error={phraseStore.error}
          onRetry={phraseStore.currentPhrase?.trim() ? handlePhraseRetry : undefined}
        />
      )}

      {showZhEnUI && <ZhEnResultsView />}

      {showPhraseUI && <PhraseCard />}

      {!showZhEnUI && !showPhraseUI && isWordLoading && (
        <div className="space-y-4">
          <Card className="p-6 define-card define-card--accent-blue">
            {progressiveWordStore.headerSection ? (
              <Section1HeaderView
                data={{
                  ...progressiveWordStore.headerSection,
                  word: isTyping ? displayedWord : progressiveWordStore.headerSection.word,
                }}
              />
            ) : (
              <Section1HeaderSkeleton />
            )}
          </Card>

          {progressiveWordStore.hasHeaderSection && (
            <Card className="p-6 animate-fadeIn define-card define-card--accent-amber">
              {progressiveWordStore.meaningsSection ? (
                <Section2MeaningsView 
                  data={progressiveWordStore.meaningsSection} 
                  word={progressiveWordStore.headerSection?.word || ''}
                />
              ) : (
                <Section2MeaningsSkeleton />
              )}
            </Card>
          )}

          {progressiveWordStore.hasHeaderSection && (
            <Card className="p-6 animate-fadeIn define-card define-card--accent-plum">
              {progressiveWordStore.relatedSection ? (
                <WordSection3RelatedView
                  data={progressiveWordStore.relatedSection}
                  onWordNavigate={(word) => { searchCoordinator.search(word, { source: 'related-word' }); }}
                />
              ) : (
                <WordSection3RelatedSkeleton />
              )}
            </Card>
          )}
        </div>
      )}

      {!showZhEnUI && !showPhraseUI && !isWordLoading && hasWordData && (
        <div className="space-y-4">
          {progressiveWordStore.headerSection && (
            <Card className="p-6 define-card define-card--accent-blue">
              <Section1HeaderView data={progressiveWordStore.headerSection} />
            </Card>
          )}

          {progressiveWordStore.meaningsSection && (
            <Card className="p-6 define-card define-card--accent-amber">
              <Section2MeaningsView 
                data={progressiveWordStore.meaningsSection}
                word={progressiveWordStore.headerSection?.word || ''}
              />
            </Card>
          )}

          {progressiveWordStore.relatedSection && (
            <Card className="p-6 define-card define-card--accent-plum">
              <WordSection3RelatedView
                data={progressiveWordStore.relatedSection}
                onWordNavigate={(word) => { searchCoordinator.search(word, { source: 'related-word' }); }}
              />
            </Card>
          )}
        </div>
      )}
    </div>
  );
});
