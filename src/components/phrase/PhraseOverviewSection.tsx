import type { PhraseSection1Overview } from '../../types';
import { useTranslation } from 'react-i18next';
import { Volume2, Share2, Lightbulb, BookOpen, MessageCircle } from 'lucide-react';
import { toast } from 'sonner';
import { PhraseTypeBadge, RegionBadge, FormalityBadge } from './PhraseTypeBadge';
import { ExportPhraseDialog } from './ExportPhraseDialog';
import { pronunciation } from '../../services/pronunciation';

interface Props {
  data: PhraseSection1Overview;
}

export const PhraseOverviewSection = ({ data }: Props) => {
  const { t } = useTranslation();

  const handleShare = async () => {
    const shareData = {
      title: `${t('phrase.shareTitle')}: ${data.phrase}`,
      text: `${data.phrase}\n\n${data.tldr}\n\n${t('phrase.meaning')}: ${data.actualMeaning}`,
    };

    if (navigator.share) {
      try {
        await navigator.share(shareData);
      } catch (error) {
        if (!(error instanceof DOMException && error.name === 'AbortError')) {
          toast.error(t('phrase.copyFailed'));
        }
      }
    } else {
      try {
        await navigator.clipboard.writeText(shareData.text);
        toast.success(t('phrase.copiedToClipboard'));
      } catch (error) {
        toast.error(t('phrase.copyFailed'));
      }
    }
  };

  return (
    <div className="space-y-6">
      <h1 className="text-4xl font-bold tracking-tight leading-tight">
        {data.phrase}
      </h1>

      <div className="flex flex-wrap items-center gap-2 pb-6 border-b border-border">
        <PhraseTypeBadge type={data.phraseType} />
        <RegionBadge region={data.region} />
        <FormalityBadge level={data.formality.level} />
      </div>

      <div className="bg-muted rounded-lg p-4">
        <div className="flex items-center gap-1.5 text-[0.6875rem] font-bold text-muted-foreground uppercase tracking-widest mb-2">
          <Lightbulb className="w-3.5 h-3.5" />
          {t('phrase.inANutshell')}
        </div>
        <p className="definition-content text-base leading-relaxed">
          {data.tldr}
        </p>
      </div>

      {data.literalMeaning && (
        <div className="space-y-4">
          <div>
            <div className="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
              <BookOpen className="w-3.5 h-3.5" />
              {t('phrase.literalMeaning')}
            </div>
            <div className="meaning-text pl-4 border-l-2 border-border text-base text-foreground leading-relaxed">
              {data.literalMeaning}
            </div>
          </div>
          <div>
            <div className="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
              <MessageCircle className="w-3.5 h-3.5" />
              {t('phrase.actualMeaning')}
            </div>
            <div className="meaning-text pl-4 border-l-2 border-primary text-base text-foreground leading-relaxed">
              {data.actualMeaning}
            </div>
          </div>
        </div>
      )}

      {!data.literalMeaning && (
        <div>
          <div className="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
            <MessageCircle className="w-3.5 h-3.5" />
            {t('phrase.meaning')}
          </div>
          <div className="meaning-text pl-4 border-l-2 border-primary text-base text-foreground leading-relaxed">
            {data.actualMeaning}
          </div>
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <button
          onClick={() => { void pronunciation.play(data.phrase); }}
          className="flex items-center gap-2 px-4 py-2 text-sm font-medium border border-border rounded-lg hover:bg-accent hover:border-foreground transition-colors cursor-pointer"
        >
          <Volume2 className="w-4 h-4" />
          {t('phrase.pronounce')}
        </button>
        <ExportPhraseDialog />
        <button
          onClick={handleShare}
          className="flex items-center gap-2 px-4 py-2 text-sm font-medium border border-border rounded-lg hover:bg-accent hover:border-foreground transition-colors cursor-pointer"
        >
          <Share2 className="w-4 h-4" />
          {t('common.share')}
        </button>
      </div>
    </div>
  );
};
