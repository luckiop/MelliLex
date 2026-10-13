import { useEffect, useMemo, useState } from 'react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { useStores } from '../stores/RootStore';
import type { AiModel, AiProviderType, TypographyOption } from '../types';
import { LANGUAGE_MAP } from '../i18n';
import { toast } from 'sonner';
import { 
  Sparkles, 
  Database, 
  Info,
  Share2,
  Palette,
  Languages,
  Keyboard,
  BookMarked,
  AudioLines,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { PageHint } from '../components/onboarding/PageHintBanner';
import { 
  AIProviderSection,
  AppearanceSection,
  CacheSection,
  ExportSection,
  LanguageSection,
  UILanguageSection,
  GlobalLookupSection,
  AboutSection,
  VocabSection,
  TechnicalDomainSection,
  VoiceSection,
} from '../components/settings';
import { DEFAULT_TECHNICAL_DOMAIN } from '../components/settings/TechnicalDomainSection';

interface SettingsSection {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
}

export const Settings = observer(() => {
  const { t, i18n } = useTranslation();
  const { settingsStore } = useStores();
  const [activeSection, setActiveSection] = useState('general');
  
  const sections: SettingsSection[] = useMemo(() => [
    {
      id: 'general',
      title: t('settings.general.title'),
      description: t('settings.general.description'),
      icon: <Languages className="w-5 h-5" />
    },
    {
      id: 'global-lookup',
      title: t('settings.globalLookup.title'),
      description: t('settings.globalLookup.description'),
      icon: <Keyboard className="w-5 h-5" />
    },
    {
      id: 'ai-provider',
      title: t('settings.aiProvider.title'),
      description: t('settings.aiProvider.description'),
      icon: <Sparkles className="w-5 h-5" />
    },
    {
      id: 'appearance',
      title: t('settings.appearance.title'),
      description: t('settings.appearance.description'),
      icon: <Palette className="w-5 h-5" />
    },
    {
      id: 'voice',
      title: t('settings.voice.title'),
      description: t('settings.voice.description'),
      icon: <AudioLines className="w-5 h-5" />
    },
    {
      id: 'cache',
      title: t('settings.cache.title'),
      description: t('settings.cache.description'),
      icon: <Database className="w-5 h-5" />
    },
    {
      id: 'export',
      title: t('settings.export.title'),
      description: t('settings.export.description'),
      icon: <Share2 className="w-5 h-5" />
    },
    {
      id: 'vocab',
      title: t('vocab.settingsTitle'),
      description: t('vocab.settingsDescription'),
      icon: <BookMarked className="w-5 h-5" />
    },
    {
      id: 'about',
      title: t('settings.about.title'),
      description: t('settings.about.description'),
      icon: <Info className="w-5 h-5" />
    }
  ], [t]);
  
  const [selectedProvider, setSelectedProvider] = useState<AiProviderType>(settingsStore.settings.aiProvider || 'anthropic');
  const [apiKey, setApiKey] = useState('');
  const [modelId, setModelId] = useState('');
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [availableModels, setAvailableModels] = useState<AiModel[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [apiKeyValid, setApiKeyValid] = useState<boolean | null>(null);
  
  const [selectedFontOption, setSelectedFontOption] = useState<TypographyOption>('classic');
  const [appliedFontOption, setAppliedFontOption] = useState<TypographyOption>('classic');
  const [applyingTypography, setApplyingTypography] = useState(false);

  const [selectedUILanguage, setSelectedUILanguage] = useState('English');
  const [selectedExplanationLanguage, setSelectedExplanationLanguage] = useState('English');
  const [applyingLanguages, setApplyingLanguages] = useState(false);

  const [globalLookupEnabled, setGlobalLookupEnabled] = useState(true);
  const [clipboardClickSearch, setClipboardClickSearch] = useState(true);
  const [technicalDomain, setTechnicalDomain] = useState(DEFAULT_TECHNICAL_DOMAIN);

  useEffect(() => {
    settingsStore.loadSettings();
    settingsStore.detectOllama();
  }, [settingsStore]);

  useEffect(() => {
    setSelectedUILanguage(settingsStore.settings.uiLanguage || 'English');
    setSelectedExplanationLanguage(settingsStore.settings.explanationLanguage || 'English');
    setGlobalLookupEnabled(settingsStore.settings.enableGlobalLookup ?? true);
    setClipboardClickSearch(settingsStore.settings.clipboardClickSearch !== false);
    setTechnicalDomain(settingsStore.settings.technicalDomain?.trim() || DEFAULT_TECHNICAL_DOMAIN);
  }, [
    settingsStore.settings.uiLanguage,
    settingsStore.settings.explanationLanguage,
    settingsStore.settings.enableGlobalLookup,
    settingsStore.settings.clipboardClickSearch,
    settingsStore.settings.technicalDomain,
  ]);

  useEffect(() => {
    const preference = (settingsStore.settings.typographyMode as TypographyOption) || 'classic';
    setSelectedFontOption(preference);
    setAppliedFontOption(preference);
  }, [settingsStore.settings.typographyMode]);

  useEffect(() => {
    const provider = settingsStore.settings.aiProvider;
    setSelectedProvider(provider || 'anthropic');
    let savedKey = '';
    
    if (provider === 'deepseek') {
      savedKey = settingsStore.settings.deepseekConfig?.apiKey || '';
      setApiKey(savedKey);
      setModelId(settingsStore.settings.deepseekConfig?.model || 'deepseek-v4-flash');
    } else if (provider === 'openai' && settingsStore.settings.openAiConfig) {
      savedKey = settingsStore.settings.openAiConfig.apiKey || '';
      setApiKey(savedKey);
      setModelId(settingsStore.settings.openAiConfig.model || 'gpt-5-mini');
    } else if (provider === 'anthropic' && settingsStore.settings.anthropicConfig) {
      savedKey = settingsStore.settings.anthropicConfig.apiKey || '';
      setApiKey(savedKey);
      setModelId(settingsStore.settings.anthropicConfig.model || 'claude-haiku-4-5-20251001');
    } else if (provider === 'gemini' && settingsStore.settings.geminiConfig) {
      savedKey = settingsStore.settings.geminiConfig.apiKey || '';
      setApiKey(savedKey);
      setModelId(settingsStore.settings.geminiConfig.model || 'gemini-2.5-flash-lite');
    } else if (provider === 'ollama') {
      setApiKey('');
      if (settingsStore.settings.ollamaConfig?.model) {
        setModelId(settingsStore.settings.ollamaConfig.model);
      } else {
        setModelId('');
      }
    } else {
      setApiKey('');
      setModelId('');
    }
    
    setApiKeyValid(null);
    setAvailableModels([]);
  }, [settingsStore.settings.aiProvider, settingsStore.settings.openAiConfig, settingsStore.settings.deepseekConfig, settingsStore.settings.anthropicConfig, settingsStore.settings.geminiConfig, settingsStore.settings.ollamaConfig]);

  const handleApplyLanguages = async () => {
    setApplyingLanguages(true);
    try {
      const languageCode = LANGUAGE_MAP[selectedUILanguage] || 'en';
      
      await settingsStore.updateSettings({
        uiLanguage: selectedUILanguage,
        explanationLanguage: selectedExplanationLanguage,
      });
      await i18n.changeLanguage(languageCode);
      setApplyingLanguages(false);
      toast.success(t('settings.general.languageApplied'));
    } catch (error) {
      setApplyingLanguages(false);
    }
  };

  const handleGlobalLookupEnabledChange = async (enabled: boolean) => {
    setGlobalLookupEnabled(enabled);
    await settingsStore.updateSettings({ enableGlobalLookup: enabled });
  };

  const handleClipboardClickSearchChange = async (enabled: boolean) => {
    setClipboardClickSearch(enabled);
    await settingsStore.updateSettings({ clipboardClickSearch: enabled });
  };

  const handleTechnicalDomainChange = async (domain: string) => {
    setTechnicalDomain(domain);
    await settingsStore.updateSettings({ technicalDomain: domain });
  };

  const handleApplyTypography = async (option: TypographyOption) => {
    setApplyingTypography(true);
    try {
      await settingsStore.updateSettings({ typographyMode: option });
      setAppliedFontOption(option);
    } catch (error) {
      console.error('Failed to apply typography preference:', error);
      throw error;
    } finally {
      setApplyingTypography(false);
    }
  };

  return (
    <div className="container mx-auto px-8 py-12 max-w-3xl">
      <PageHint pageId="settings" />
      <div className="mb-8">
        <h1 className="text-4xl font-bold text-foreground mb-6">{t('settings.title')}</h1>
        <div className="flex gap-1 border-b border-border pb-2 overflow-x-auto">
          {sections.map((section) => (
            <button
              key={section.id}
              onClick={() => setActiveSection(section.id)}
              className={`flex items-center gap-1 px-2 py-1 rounded-md transition-colors text-xs font-medium whitespace-nowrap ${
                activeSection === section.id
                  ? 'bg-primary text-primary-foreground'
                  : 'text-foreground hover:bg-accent'
              }`}
            >
              {section.icon}
              <span>{section.title}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="space-y-6">
        {sections.map((section) => {
          if (section.id !== activeSection) return null;

          return (
            <div key={section.id}>
              {settingsStore.error && (
                <div className="mb-6 rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-sm text-destructive">
                  {settingsStore.error}
                </div>
              )}

              <div className="bg-card rounded-lg border border-border p-6">
                {section.id === 'general' && (
                  <div className="space-y-6">
                    <div className="space-y-4">
                      <UILanguageSection 
                        value={selectedUILanguage}
                        onChange={setSelectedUILanguage}
                      />
                      <LanguageSection 
                        value={selectedExplanationLanguage}
                        onChange={setSelectedExplanationLanguage}
                      />
                    </div>
                    
                    <div className="bg-muted p-3 rounded-md text-xs text-muted-foreground">
                      <p>{t('settings.general.noteEnglishWords')}</p>
                    </div>

                    <TechnicalDomainSection
                      value={technicalDomain}
                      onChange={(domain) => {
                        void handleTechnicalDomainChange(domain);
                      }}
                    />

                    <Button
                      onClick={handleApplyLanguages}
                      disabled={
                        (selectedUILanguage === (settingsStore.settings.uiLanguage || 'English') &&
                         selectedExplanationLanguage === (settingsStore.settings.explanationLanguage || 'English')) ||
                        applyingLanguages
                      }
                      className="w-full"
                    >
                      {applyingLanguages 
                        ? t('common.applying') 
                        : (selectedUILanguage === (settingsStore.settings.uiLanguage || 'English') &&
                           selectedExplanationLanguage === (settingsStore.settings.explanationLanguage || 'English'))
                        ? t('common.applied')
                        : t('common.apply')
                      }
                    </Button>
                  </div>
                )}
                {section.id === 'global-lookup' && (
                  <GlobalLookupSection
                    enabled={globalLookupEnabled}
                    onEnabledChange={handleGlobalLookupEnabledChange}
                    clipboardSearchEnabled={clipboardClickSearch}
                    onClipboardSearchChange={(enabled) => {
                      void handleClipboardClickSearchChange(enabled);
                    }}
                  />
                )}
                {section.id === 'ai-provider' && (
                  <AIProviderSection
                    selectedProvider={selectedProvider}
                    setSelectedProvider={setSelectedProvider}
                    apiKey={apiKey}
                    setApiKey={setApiKey}
                    modelId={modelId}
                    setModelId={setModelId}
                    saveStatus={saveStatus}
                    setSaveStatus={setSaveStatus}
                    availableModels={availableModels}
                    setAvailableModels={setAvailableModels}
                    loadingModels={loadingModels}
                    setLoadingModels={setLoadingModels}
                    apiKeyValid={apiKeyValid}
                    setApiKeyValid={setApiKeyValid}
                  />
                )}
                {section.id === 'appearance' && (
                  <AppearanceSection
                    selectedFontOption={selectedFontOption}
                    setSelectedFontOption={setSelectedFontOption}
                    appliedFontOption={appliedFontOption}
                    isApplyingTypography={applyingTypography}
                    onApplyTypography={handleApplyTypography}
                  />
                )}
                {section.id === 'voice' && <VoiceSection />}
                {section.id === 'cache' && <CacheSection />}
                {section.id === 'export' && <ExportSection />}
                {section.id === 'vocab' && <VocabSection />}
                {section.id === 'about' && <AboutSection />}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
});
