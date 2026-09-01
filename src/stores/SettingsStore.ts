import { makeAutoObservable, runInAction } from 'mobx';
import { api } from '../services/api';
import type {
  AppSettings,
  AiProviderType,
  CapacitiesConfig,
  ExportSettings,
  TypographyOption,
} from '../types';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';
import { applyTypographyOption, DEFAULT_TYPOGRAPHY_OPTION } from '../utils/typography';
import { ExportSettingsStore } from './ExportSettingsStore';

const createDefaultExportSettings = (): ExportSettings => ({
  includeExploration: false,
  capacities: {
    apiToken: '',
    spaceId: '',
    defaultTags: [],
    noTimestamp: false,
  },
});

export class SettingsStore {
  settings: AppSettings = {
    aiProvider: 'anthropic',
    theme: 'system',
    exportSettings: createDefaultExportSettings(),
    enableGlobalLookup: true,
    typographyMode: DEFAULT_TYPOGRAPHY_OPTION,
    technicalQuery: false,
    vocabNoteEnabled: false,
    vocabNotePath: '',
  };
  loading: boolean = false;
  error: string | null = null;
  ollamaDetected: boolean = false;
  ollamaModels: string[] = [];

  exportSettingsStore: ExportSettingsStore;

  constructor() {
    makeAutoObservable(this);
    
    this.exportSettingsStore = new ExportSettingsStore({
      onUpdate: async (exportSettings: ExportSettings) => {
        await this.updateSettings({ exportSettings });
      },
    });
  }

  async loadSettings() {
    this.loading = true;
    this.error = null;

    try {
      const settings = await api.getSettings();
      
      runInAction(() => {
        this.settings = {
          ...this.settings,
          ...settings,
          exportSettings: mergeExportSettings(
            settings.exportSettings,
            this.settings.exportSettings ?? createDefaultExportSettings()
          ),
        };
        this.loading = false;
        this.applyTypography(this.settings.typographyMode);
        
        this.exportSettingsStore.loadFromSettings(this.settings.exportSettings);
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.LOAD_SETTINGS_FAILED);
        this.loading = false;
      });
    }
  }

  async updateSettings(settings: Partial<AppSettings>) {
    const updated = {
      ...this.settings,
      ...settings,
      exportSettings: mergeExportSettings(
        settings.exportSettings ?? this.settings.exportSettings,
        createDefaultExportSettings()
      ),
    };

    try {
      this.error = null;
      await api.updateSettings(updated);
      
      runInAction(() => {
        this.settings = updated;
        this.applyTypography(updated.typographyMode);
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.UPDATE_SETTINGS_FAILED);
      });
    }
  }

  async updateProvider(provider: AiProviderType, config: any) {
    try {
      this.error = null;
      await api.updateAiProvider(provider, config);
      
      runInAction(() => {
        this.settings.aiProvider = provider;
        if (provider === 'openai') this.settings.openAiConfig = config;
        if (provider === 'anthropic') this.settings.anthropicConfig = config;
        if (provider === 'gemini') this.settings.geminiConfig = config;
        if (provider === 'ollama') this.settings.ollamaConfig = config;
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.UPDATE_PROVIDER_FAILED);
      });
    }
  }

  async detectOllama() {
    try {
      const detected = await api.detectOllama();
      
      runInAction(() => {
        this.ollamaDetected = detected;
      });

      if (detected) {
        await this.loadOllamaModels();
      }
    } catch (err) {
      runInAction(() => {
        this.ollamaDetected = false;
        // Don't overwrite existing errors
        if (!this.error) {
          this.error = getErrorMessage(err, 'Failed to detect Ollama');
        }
      });
    }
  }

  async loadOllamaModels() {
    try {
      const models = await api.listOllamaModels();
      
      runInAction(() => {
        this.ollamaModels = models;
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.LOAD_MODELS_FAILED);
      });
    }
  }

  get isCustomProvider() {
    return ['openai', 'anthropic', 'gemini'].includes(this.settings.aiProvider);
  }

  get isLocalProvider() {
    return this.settings.aiProvider === 'ollama';
  }

  private applyTypography(mode?: TypographyOption) {
    applyTypographyOption(mode ?? DEFAULT_TYPOGRAPHY_OPTION);
  }
}

const mergeExportSettings = (
  incoming?: ExportSettings,
  fallback?: ExportSettings
): ExportSettings => {
  const includeExploration =
    incoming?.includeExploration ?? fallback?.includeExploration ?? false;

  const capacities = normalizeCapacities(incoming?.capacities ?? fallback?.capacities);

  return {
    includeExploration,
    capacities,
  };
};

const normalizeCapacities = (
  source?: CapacitiesConfig | null
): CapacitiesConfig | undefined => {
  if (!source) {
    return undefined;
  }

  const tags = Array.isArray(source.defaultTags) ? source.defaultTags : [];

  return {
    apiToken: source.apiToken ?? '',
    spaceId: source.spaceId ?? '',
    defaultTags: tags,
    noTimestamp: source.noTimestamp ?? false,
  };
};
