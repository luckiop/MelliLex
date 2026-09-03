import { useTranslation } from 'react-i18next';
import { Button } from '../ui/button';
import type { TypographyOption } from '../../types';
import { TYPOGRAPHY_PRESETS } from '../../utils/typography';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { useStores } from '../../stores/RootStore';

interface AppearanceSectionProps {
  selectedFontOption: TypographyOption;
  setSelectedFontOption: (option: TypographyOption) => void;
  appliedFontOption: TypographyOption;
  isApplyingTypography: boolean;
  onApplyTypography: (option: TypographyOption) => Promise<void> | void;
}

export const AppearanceSection = ({
  selectedFontOption,
  setSelectedFontOption,
  appliedFontOption,
  isApplyingTypography,
  onApplyTypography,
}: AppearanceSectionProps) => {
  const { t } = useTranslation();
  const { settingsStore } = useStores();
  const previewFonts = TYPOGRAPHY_PRESETS[selectedFontOption];
  const maxMeanings = String(settingsStore.settings.maxMeaningsDisplay ?? 5);

  const handleApplyClick = async () => {
    await onApplyTypography(selectedFontOption);
  };

  const handleMaxMeanings = async (value: string) => {
    const parsed = Number(value);
    if (parsed === 3 || parsed === 5 || parsed === 8) {
      await settingsStore.updateSettings({ maxMeaningsDisplay: parsed });
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-foreground mb-4">{t('settings.appearance.typography')}</h3>
        <p className="text-sm text-muted-foreground mb-6">
          {t('settings.appearance.typographyDesc')}
        </p>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              {t('settings.appearance.fontStyle')}
            </label>
            <Select
              value={selectedFontOption}
              onValueChange={(value) => setSelectedFontOption(value as TypographyOption)}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('settings.appearance.selectFontStyle')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="modern">
                  {t('settings.appearance.modern')} - {t('settings.appearance.modernDesc')}
                </SelectItem>
                <SelectItem value="classic">
                  {t('settings.appearance.classic')} - {t('settings.appearance.classicDesc')}
                </SelectItem>
                <SelectItem value="friendly">
                  {t('settings.appearance.friendly')} - {t('settings.appearance.friendlyDesc')}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

        <div className="mt-4 p-4 bg-muted rounded-lg border border-border">
          <div className="text-xs font-medium text-muted-foreground mb-3">{t('settings.appearance.preview')}</div>
          <div style={{ fontFamily: previewFonts.contentFont }}>
            <div className="text-lg font-semibold mb-2">{t('settings.appearance.previewWord')}</div>
            <div className="text-sm leading-relaxed">
              {t('settings.appearance.previewDescription')}
            </div>
          </div>
        </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold text-foreground mb-2">{t('settings.appearance.maxMeanings')}</h3>
        <p className="text-sm text-muted-foreground mb-4">
          {t('settings.appearance.maxMeaningsDesc')}
        </p>
        <Select value={maxMeanings} onValueChange={(value) => void handleMaxMeanings(value)}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="3">3</SelectItem>
            <SelectItem value="5">5</SelectItem>
            <SelectItem value="8">8</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <Button
        onClick={handleApplyClick}
        disabled={selectedFontOption === appliedFontOption || isApplyingTypography}
      >
        {isApplyingTypography
          ? t('common.applying')
          : selectedFontOption === appliedFontOption
          ? t('common.applied')
          : t('common.apply')}
      </Button>
    </div>
  );
};
