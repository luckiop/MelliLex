import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { APP_VERSION } from '../constants/app';
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
import { TranslateResultsView } from '../components/translate/TranslateResultsView';
import { HomeFeatureGrid } from '../components/onboarding/HomeFeatureGrid';
import { PageHint } from '../components/onboarding/PageHintBanner';

export const Home = observer(() => {
  const { t } = useTranslation();
  const { progressiveWordStore, phraseStore, zhEnStore, translateStore, searchCoordinator, lastPageStore } = useStores();
  const [appVersion, setAppVersion] = useState(APP_VERSION);

  useEffect(() => {
    lastPageStore.setLastPage('/');
  }, [lastPageStore]);

  useEffect(() => {
    void getVersion()
      .then(setAppVersion)
      .catch(() => setAppVersion(APP_VERSION));
  }, []);

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
  const hasTranslateData =
    Boolean(translateStore.result) || translateStore.isLoading || translateStore.hasError;
  const hasAnyData = hasWordData || hasPhraseData || hasZhEnData || hasTranslateData;
  const isWordLoading = progressiveWordStore.isLoading;
  const isPhraseLoading = phraseStore.isLoading;
  const isLoading = isWordLoading || isPhraseLoading || zhEnStore.isLoading || translateStore.isLoading;
  
  const showTranslateUI =
    hasTranslateData ||
    (translateStore.isLoading && searchCoordinator.currentInputType === 'translate');
  const showZhEnUI =
    !showTranslateUI &&
    (hasZhEnData ||
      (zhEnStore.isLoading && searchCoordinator.currentInputType === 'zh-en'));
  const showPhraseUI =
    !showTranslateUI &&
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
      {!hasAnyData && !isLoading && !progressiveWordStore.hasError && !phraseStore.hasError && !zhEnStore.hasError && !translateStore.hasError && (
        <div className="mx-auto max-w-xl py-10 text-center">
          <h1 className="mb-3 text-5xl font-semibold tracking-tight text-foreground">
            {t('home.title')}
          </h1>
          <p className="text-lg text-muted-foreground">
            {t('home.subtitle')}
          </p>
          {appVersion ? (
            <p className="mb-5 mt-2 text-xs text-muted-foreground/80">
              {t('home.versionLabel', { version: appVersion })}
            </p>
          ) : null}
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

      {showTranslateUI && <TranslateResultsView />}

      {showZhEnUI && <ZhEnResultsView />}

      {showPhraseUI && <PhraseCard />}

      {!showTranslateUI && !showZhEnUI && !showPhraseUI && isWordLoading && (
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
              {hasRelatedContent(progressiveWordStore.relatedSection) ? (
                <WordSection3RelatedView
                  data={progressiveWordStore.relatedSection!}
                  onWordNavigate={(word) => { searchCoordinator.search(word, { source: 'related-word' }); }}
                />
              ) : (
                <WordSection3RelatedSkeleton />
              )}
            </Card>
          )}
        </div>
      )}

      {!showTranslateUI && !showZhEnUI && !showPhraseUI && !isWordLoading && hasWordData && (
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

          {hasRelatedContent(progressiveWordStore.relatedSection) && (
            <Card className="p-6 define-card define-card--accent-plum">
              <WordSection3RelatedView
                data={progressiveWordStore.relatedSection!}
                onWordNavigate={(word) => { searchCoordinator.search(word, { source: 'related-word' }); }}
              />
            </Card>
          )}
        </div>
      )}

      {appVersion && (hasAnyData || isLoading) ? (
        <p className="mt-12 text-center text-xs text-muted-foreground/70 font-mono">
          {t('home.versionLabel', { version: appVersion })}
        </p>
      ) : null}
    </div>
  );
});

function hasRelatedContent(
  related: { synonyms: string[]; antonyms: string[]; collocations: unknown[] } | null
): boolean {
  if (!related) {
    return false;
  }
  return (
    related.synonyms.length > 0 ||
    related.antonyms.length > 0 ||
    related.collocations.length > 0
  );
}
