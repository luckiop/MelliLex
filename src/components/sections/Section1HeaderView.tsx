import type { WordSection1Header } from '../../types';
import { Volume2, Eye, Share2, Globe, Layers, Cpu } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { observer } from 'mobx-react-lite';
import { toast } from 'sonner';
import { ExportWordDialog } from '../ExportWordDialog';
import { useTranslation } from 'react-i18next';
import { useStores } from '../../stores/RootStore';
import { snippetsFromMeanings } from '../../utils/vocab';
import { VocabStarIcons } from '../vocab/VocabStarIcons';
import { pronunciation } from '../../services/pronunciation';

interface Props {
  data: WordSection1Header;
}

export const Section1HeaderView = observer(({ data }: Props) => {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const { progressiveWordStore, settingsStore, vocabStore } = useStores();
  const lookupWord = progressiveWordStore.currentWord || data.word;

  useEffect(() => {
    const defaultCategory = settingsStore.settings.technicalQuery ? 'technical' : 'daily';
    void vocabStore.loadForWord(lookupWord, defaultCategory);
  }, [lookupWord, settingsStore.settings.technicalQuery, vocabStore]);

  const getOriginDisplay = (origin: string) => {
    const firstWord = origin.split(' ')[0];
    return { display: firstWord, full: origin };
  };

  const originInfo = getOriginDisplay(data.origin);

  const handleExplore = () => {
    navigate('/explore');
  };

  const handleShare = async () => {
    const shareData = {
      title: `Word: ${data.word}`,
      text: `${data.word} (${data.pronunciation})\n\n${data.tldr}`,
    };

    if (navigator.share) {
      try {
        await navigator.share(shareData);
      } catch (error) {
        if (!(error instanceof DOMException && error.name === 'AbortError')) {
          toast.error(t('wordHeader.failedToCopy'));
        }
      }
    } else {
      try {
        await navigator.clipboard.writeText(shareData.text);
        toast.success(t('wordHeader.copiedToClipboard'));
      } catch (error) {
        toast.error(t('wordHeader.failedToCopy'));
      }
    }
  };

  const handleCycleStars = async () => {
    try {
      const notes = snippetsFromMeanings(
        progressiveWordStore.meaningsSection?.meanings,
        data.tldr
      );
      const result = await vocabStore.cycleStars(lookupWord, notes);
      if (result?.noteError) {
        toast.error(result.noteError);
      } else if (result?.noteAppended) {
        toast.success(t('vocab.noteAppended'));
      }
    } catch {
      toast.error(t('vocab.updateFailed'));
    }
  };

  const handleToggleCategory = async () => {
    try {
      await vocabStore.toggleCategory(lookupWord);
    } catch {
      toast.error(t('vocab.updateFailed'));
    }
  };

  const stars = vocabStore.currentEntry?.stars ?? 0;
  const category = vocabStore.activeCategory;

  return (
    <div className="space-y-6">
      <h1 className="text-5xl font-bold tracking-tight leading-none">
        {data.word}
      </h1>

      <div className="flex items-center gap-3 pb-6 border-b border-border">
        <button
          onClick={() => { void pronunciation.play(data.word); }}
          className="p-2 hover:bg-accent rounded-lg transition-colors border border-border hover:border-foreground"
          aria-label={t('wordHeader.pronounceAriaLabel')}
        >
          <Volume2 className="w-4 h-4" />
        </button>
        <span className="text-base font-mono text-foreground font-medium">
          {data.pronunciation}
        </span>
        <span className="text-sm text-muted-foreground">·</span>
        <span className="text-sm text-muted-foreground">{data.syllables}</span>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-muted rounded-lg flex-shrink-0">
            <Globe className="w-4 h-4 text-muted-foreground" />
          </div>
          <div className="flex flex-col gap-0.5 min-w-0">
            <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
              {t('wordHeader.origin')}
            </div>
            <div 
              className="text-base font-medium cursor-help truncate" 
              title={originInfo.full}
            >
              {originInfo.display}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className="p-2 bg-muted rounded-lg flex-shrink-0">
            <Layers className="w-4 h-4 text-muted-foreground" />
          </div>
          <div className="flex flex-col gap-0.5 min-w-0">
            <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
              {t('wordHeader.formality')}
            </div>
            <div className="text-base font-medium truncate">
              {data.formality.level} · {data.formality.percentage}%
            </div>
          </div>
        </div>
      </div>

      {data.domains.length > 0 && (
        <div className="flex items-center gap-3 flex-wrap">
          <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            {t('wordHeader.usedIn')}
          </span>
          {data.domains.map((domain, i) => (
            <span
              key={i}
              className="px-3 py-1.5 bg-muted text-sm font-medium rounded-full border border-border hover:border-muted-foreground/50 transition-colors"
            >
              {domain}
            </span>
          ))}
        </div>
      )}

      <div className="bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-600 p-4 rounded">
        <div className="text-xs font-bold text-blue-900 dark:text-blue-200 uppercase tracking-wider mb-2">
          {t('sections.tldr')}
        </div>
        <div className="definition-content text-base text-blue-900 dark:text-blue-100 leading-relaxed">
          {data.tldr}
        </div>
      </div>

      <div className="flex gap-2 flex-wrap pt-2">
        <button
          type="button"
          onClick={handleCycleStars}
          aria-pressed={stars > 0}
          aria-label={
            stars === 0 ? t('vocab.add') : stars === 1 ? t('vocab.oneStar') : t('vocab.twoStars')
          }
          title={t('vocab.starHint')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-all ${
            stars > 0
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border bg-card hover:bg-accent hover:border-foreground'
          }`}
        >
          <VocabStarIcons count={stars} className="w-4 h-4" />
          {stars === 0 ? t('vocab.add') : null}
        </button>
        <button
          type="button"
          onClick={handleToggleCategory}
          title={t('vocab.categoryHint')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-all ${
            category === 'technical'
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border bg-card hover:bg-accent hover:border-foreground'
          }`}
        >
          <Cpu className="w-4 h-4" />
          {category === 'technical' ? t('vocab.technical') : t('vocab.daily')}
        </button>
        <button 
          onClick={handleExplore}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90 transition-all text-sm font-medium"
        >
          <Eye className="w-4 h-4" />
          {t('common.explore')}
        </button>
        <button 
          onClick={handleShare}
          className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border bg-card hover:bg-accent hover:border-foreground transition-all text-sm font-medium"
        >
          <Share2 className="w-4 h-4" />
          {t('common.share')}
        </button>
        <ExportWordDialog />
      </div>
    </div>
  );
});
