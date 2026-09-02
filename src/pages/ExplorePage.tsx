import { useState, useEffect } from 'react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { useStores } from '../stores/RootStore';
import { Button } from '../components/ui/button';
import { Card } from '../components/ui/card';
import { ArrowLeft, Search } from 'lucide-react';
import { FormalityCard } from '../components/explore/FormalityCard';
import { DomainCard } from '../components/explore/DomainCard';
import { UsageCard } from '../components/explore/UsageCard';
import { MistakesCard } from '../components/explore/MistakesCard';
import { CustomContextCard } from '../components/explore/CustomContextCard';
import { PracticeCard } from '../components/explore/PracticeCard';
import { PageHint } from '../components/onboarding/PageHintBanner';

type SectionId = 'formality' | 'domains' | 'usage' | 'mistakes' | 'custom' | 'practice';

const ExplorePage = observer(() => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { exploreStore, lastPageStore } = useStores();
  const [activeSection, setActiveSection] = useState<SectionId>('formality');

  useEffect(() => {
    lastPageStore.setLastPage('/explore');
  }, [lastPageStore]);

  const currentWord = exploreStore.currentWord;
  const {
    formalityState,
    formalityPercentage,
    formalityAlternatives,
    formalityError,
    domainsState,
    domainExplorations,
    domainsError,
    usageState,
    usagePatterns,
    usageError,
    mistakesState,
    commonMistakes,
    mistakesError,
    customContextState,
    customExamples,
    customContext,
    customContextError,
    practiceState,
    practiceExercises,
    practiceError,
  } = exploreStore;

  const handleNavClick = (sectionId: SectionId) => {
    setActiveSection(sectionId);
  };

  const sections: Array<{ id: SectionId; label: string }> = [
    { id: 'formality', label: t('explore.formality') },
    { id: 'domains', label: t('explore.domains') },
    { id: 'usage', label: t('explore.usage') },
    { id: 'mistakes', label: t('explore.mistakes') },
    { id: 'custom', label: t('explore.custom') },
    { id: 'practice', label: t('explore.practice') },
  ];

  return (
    <div className="container mx-auto px-8 py-12 max-w-3xl">
      <PageHint pageId="explore" />
      {!currentWord ? (
        <div className="text-center py-20">
          <div className="flex justify-center mb-4">
            <Search className="h-14 w-14 text-muted-foreground" />
          </div>
          <h2 className="text-3xl font-extrabold text-foreground mb-4">{t('explore.subtitle')}</h2>
          <p className="text-lg text-muted-foreground">
            {t('explore.searchPrompt')}
          </p>
        </div>
      ) : (
        <>
      <div className="mb-8">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => navigate('/')}
          className="mb-4 gap-2"
        >
          <ArrowLeft className="w-4 h-4" />
          {t('explore.backToDefine')}
        </Button>
        <h1 className="text-4xl font-bold text-foreground mb-2">
          {currentWord}
        </h1>
        <p className="text-muted-foreground">
          {t('explore.subtitle')}
        </p>
      </div>

      <Card className="p-4 mb-4">
        <div className="flex gap-2 flex-wrap">
          {sections.map(({ id, label }) => (
            <Button
              key={id}
              variant={activeSection === id ? 'default' : 'ghost'}
              size="sm"
              onClick={() => handleNavClick(id)}
              className="whitespace-nowrap"
            >
              {label}
            </Button>
          ))}
        </div>
      </Card>

      <div className="space-y-4">
        {activeSection === 'formality' && (
          <FormalityCard
            currentWord={currentWord}
            state={formalityState}
            percentage={formalityPercentage}
            alternatives={formalityAlternatives}
            error={formalityError}
            onGenerate={() => exploreStore.generateFormality()}
          />
        )}
        {activeSection === 'domains' && (
          <DomainCard
            currentWord={currentWord}
            state={domainsState}
            domains={domainExplorations}
            error={domainsError}
            onGenerate={() => exploreStore.generateDomains()}
          />
        )}
        {activeSection === 'usage' && (
          <UsageCard
            currentWord={currentWord}
            state={usageState}
            patterns={usagePatterns}
            error={usageError}
            onGenerate={() => exploreStore.generateUsage()}
          />
        )}
        {activeSection === 'mistakes' && (
          <MistakesCard
            currentWord={currentWord}
            state={mistakesState}
            mistakes={commonMistakes}
            error={mistakesError}
            onGenerate={() => exploreStore.generateMistakes()}
          />
        )}
        {activeSection === 'custom' && (
          <CustomContextCard
            state={customContextState}
            examples={customExamples}
            contextLabel={customContext}
            error={customContextError}
            onGenerate={(context) => exploreStore.generateCustomExamples(context)}
          />
        )}
        {activeSection === 'practice' && (
          <PracticeCard
            currentWord={currentWord}
            state={practiceState}
            exercises={practiceExercises}
            error={practiceError}
            onGenerate={() => exploreStore.generatePractice()}
            onRegenerate={() => exploreStore.generatePractice(true)}
          />
        )}
      </div>
        </>
      )}
    </div>
  );
});

export default ExplorePage;
