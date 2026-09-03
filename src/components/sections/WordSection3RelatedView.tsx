import type { WordSection3Related } from '../../types';
import { useTranslation } from 'react-i18next';
import { isEnglishLexeme } from '../../utils/inputDetection';

interface Props {
  data: WordSection3Related;
  onWordNavigate?: (word: string) => void | Promise<void>;
}

export const WordSection3RelatedView = ({ data, onWordNavigate }: Props) => {
  const { t } = useTranslation();
  const synonyms = data.synonyms.filter(isEnglishLexeme);
  const antonyms = data.antonyms.filter(isEnglishLexeme);
  const collocations = data.collocations.filter((item) => isEnglishLexeme(item.phrase));
  
  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold mb-4">{t('sections.related')}</h2>
      {synonyms.length > 0 && (
        <div className="space-y-2">
          <h3 className="font-semibold text-[0.8125rem] text-muted-foreground uppercase tracking-wide mb-1">
            {t('sections.synonyms')}
          </h3>
          <div className="flex flex-wrap gap-2">
            {synonyms.map((synonym, i) => (
              <button
                type="button"
                key={i}
                onClick={() => onWordNavigate?.(synonym)}
                className="px-3 py-1.5 bg-muted border border-border rounded text-[0.875rem] cursor-pointer hover:bg-accent hover:border-foreground transition-all"
              >
                {synonym}
              </button>
            ))}
          </div>
        </div>
      )}

      {antonyms.length > 0 && (
        <div className="space-y-2">
          <h3 className="font-semibold text-[0.8125rem] text-muted-foreground uppercase tracking-wide mb-1">
            {t('sections.antonyms')}
          </h3>
          <div className="flex flex-wrap gap-2">
            {antonyms.map((antonym, i) => (
              <button
                type="button"
                key={i}
                onClick={() => onWordNavigate?.(antonym)}
                className="px-3 py-1.5 bg-muted border border-border rounded text-[0.875rem] cursor-pointer hover:bg-accent hover:border-foreground transition-all"
              >
                {antonym}
              </button>
            ))}
          </div>
        </div>
      )}

      {collocations.length > 0 && (
        <div className="space-y-2">
          <h3 className="font-semibold text-[0.8125rem] text-muted-foreground uppercase tracking-wide mb-1">
            {t('sections.collocations')}
          </h3>
          <div className="space-y-2">
            {collocations.map((collocation, i) => (
              <div
                key={i}
                className="p-2 px-3 bg-card border border-border rounded text-[0.875rem]"
              >
                <div className="definition-content font-medium mb-1">{collocation.phrase}</div>
                <div className="example-text text-[0.8125rem] text-muted-foreground italic">
                  "{collocation.example}"
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
