import { useEffect, useCallback, useState } from 'react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import type { AiProviderType, AiModel, ConnectivityReport } from '../../types';
import { api } from '../../services/api';
import { parseError } from '../../utils/errorHandler';
import { useStores } from '../../stores/RootStore';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { Activity, Check, Loader2, X, Zap } from 'lucide-react';

const RECOMMENDED_MODEL_IDS = new Set([
  'deepseek-v4-flash',
  'gpt-5-mini',         // OpenAI fastest
  'claude-haiku-4-5-20251001', // Anthropic fastest
  'gemini-2.5-flash-lite',     // Gemini fastest
]);

const CUSTOM_PROVIDERS: AiProviderType[] = ['deepseek', 'openai', 'anthropic', 'gemini'];

interface AIProviderSectionProps {
  selectedProvider: AiProviderType;
  setSelectedProvider: (provider: AiProviderType) => void;
  apiKey: string;
  setApiKey: (key: string) => void;
  modelId: string;
  setModelId: (id: string) => void;
  saveStatus: 'idle' | 'saving' | 'saved' | 'error';
  setSaveStatus: (status: 'idle' | 'saving' | 'saved' | 'error') => void;
  availableModels: AiModel[];
  setAvailableModels: (models: AiModel[]) => void;
  loadingModels: boolean;
  setLoadingModels: (loading: boolean) => void;
  apiKeyValid: boolean | null;
  setApiKeyValid: (valid: boolean | null) => void;
}

export const AIProviderSection = observer(({
  selectedProvider,
  setSelectedProvider,
  apiKey,
  setApiKey,
  modelId,
  setModelId,
  saveStatus,
  setSaveStatus,
  availableModels,
  setAvailableModels,
  loadingModels,
  setLoadingModels,
  apiKeyValid,
  setApiKeyValid,
}: AIProviderSectionProps) => {
  const { t } = useTranslation();
  const { settingsStore } = useStores();
  const [probing, setProbing] = useState(false);
  const [probeResult, setProbeResult] = useState<ConnectivityReport | null>(null);

  const providerLabel = (provider: string) => {
    switch (provider) {
      case 'deepseek': return 'DeepSeek';
      case 'openai': return 'OpenAI';
      case 'anthropic': return 'Anthropic';
      case 'gemini': return 'Gemini';
      case 'ollama': return 'Ollama';
      default: return provider;
    }
  };

  const probeSummary = (report: ConnectivityReport) => {
    const seconds = (report.elapsedMs / 1000).toFixed(1);
    const status = report.httpStatus ?? '—';
    const provider = providerLabel(report.provider);
    switch (report.reason) {
      case 'ok':
        return t('settings.aiProvider.connectivity.ok', { model: report.model, seconds });
      case 'not_configured':
        return t('settings.aiProvider.connectivity.notConfigured');
      case 'timeout_after_headers':
        return t('settings.aiProvider.connectivity.timeoutAfterHeaders', {
          provider,
          model: report.model,
          status,
          seconds,
        });
      case 'timeout':
        return t('settings.aiProvider.connectivity.timeout', { provider, seconds });
      case 'network':
        return t('settings.aiProvider.connectivity.network');
      case 'http_error':
        return t('settings.aiProvider.connectivity.httpError', { provider, status });
      case 'empty_reply':
        return t('settings.aiProvider.connectivity.emptyReply');
      case 'unsupported':
        return t('settings.aiProvider.connectivity.unsupported');
      default:
        return t('settings.aiProvider.connectivity.unknown');
    }
  };

  const isCustomProvider = CUSTOM_PROVIDERS.includes(selectedProvider);
  const isLocalProvider = selectedProvider === 'ollama';

  const handleProbeConnectivity = async () => {
    if ((isCustomProvider && !apiKey.trim()) || !modelId.trim()) {
      setProbeResult({
        ok: false,
        reason: 'not_configured',
        elapsedMs: 0,
        httpStatus: null,
        provider: selectedProvider,
        model: modelId,
        replyPreview: null,
        rawError: 'API key or model is empty',
      });
      return;
    }

    setProbing(true);
    setProbeResult(null);
    try {
      const report = await api.probeModelConnectivity(
        selectedProvider,
        isLocalProvider ? '' : apiKey,
        modelId,
      );
      setProbeResult(report);
    } catch (error) {
      const raw = parseError(error).message;
      setProbeResult({
        ok: false,
        reason: 'unknown',
        elapsedMs: 0,
        httpStatus: null,
        provider: selectedProvider,
        model: modelId,
        replyPreview: null,
        rawError: raw,
      });
    } finally {
      setProbing(false);
    }
  };

  const handleFetchModels = useCallback(async () => {
    if (!apiKey.trim()) return;
    
    setLoadingModels(true);
    setApiKeyValid(null);
    
    try {
      const models = await api.fetchAvailableModels(selectedProvider, apiKey);
      setAvailableModels(models);
      setApiKeyValid(true);
      
      if (!modelId && models.length > 0) {
        setModelId(models[0].id);
      }
    } catch (error) {
      console.error('Failed to fetch models:', error);
      setApiKeyValid(false);
    } finally {
      setLoadingModels(false);
    }
  }, [apiKey, selectedProvider, modelId, setLoadingModels, setApiKeyValid, setAvailableModels, setModelId]);

  useEffect(() => {
    if (isLocalProvider && settingsStore.ollamaModels.length > 0) {
      if (!modelId || modelId.trim() === '') {
        setModelId(settingsStore.ollamaModels[0]);
      }
    }
  }, [isLocalProvider, settingsStore.ollamaModels, setModelId]);

  useEffect(() => {
    if (apiKey.trim() && isCustomProvider) {
      const timeoutId = setTimeout(() => {
        handleFetchModels();
      }, 300);
      
      return () => clearTimeout(timeoutId);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [apiKey, isCustomProvider]);

  const handleProviderChange = (provider: AiProviderType) => {
    setSelectedProvider(provider);
    setApiKeyValid(null);
    setAvailableModels([]);
    setSaveStatus('idle');
    setProbeResult(null);

    const settings = settingsStore.settings;
    if (provider === 'deepseek') {
      setApiKey(settings.deepseekConfig?.apiKey || '');
      setModelId(settings.deepseekConfig?.model || 'deepseek-v4-flash');
    } else if (provider === 'openai' && settings.openAiConfig) {
      setApiKey(settings.openAiConfig.apiKey || '');
      setModelId(settings.openAiConfig.model || 'gpt-5-mini');
    } else if (provider === 'anthropic' && settings.anthropicConfig) {
      setApiKey(settings.anthropicConfig.apiKey || '');
      setModelId(settings.anthropicConfig.model || 'claude-haiku-4-5-20251001');
    } else if (provider === 'gemini' && settings.geminiConfig) {
      setApiKey(settings.geminiConfig.apiKey || '');
      setModelId(settings.geminiConfig.model || 'gemini-2.5-flash-lite');
    } else if (provider === 'ollama') {
      setApiKey('');
      setModelId(settings.ollamaConfig?.model || '');
    } else {
      setApiKey('');
      setModelId('');
    }
  };

  const handleSaveConfiguration = async () => {
    setSaveStatus('saving');
    try {
      if (selectedProvider === 'ollama') {
        await settingsStore.updateProvider(selectedProvider, {
          model: modelId,
          endpoint: 'http://localhost:11434',
        });
      } else {
        await settingsStore.updateProvider(selectedProvider, {
          apiKey,
          model: modelId,
        });
      }
      
      setSaveStatus('saved');
      setTimeout(() => setSaveStatus('idle'), 2000);
    } catch (error) {
      console.error('Failed to save configuration:', error);
      setSaveStatus('error');
      setTimeout(() => setSaveStatus('idle'), 3000);
    }
  };

  const getDefaultModel = (provider: AiProviderType): string => {
    switch (provider) {
      case 'deepseek': return 'deepseek-v4-flash';
      case 'openai': return 'gpt-5-mini';
      case 'anthropic': return 'claude-haiku-4-5-20251001';
      case 'gemini': return 'gemini-2.5-flash-lite';
      default: return '';
    }
  };

  const getSaveButtonText = () => {
    switch (saveStatus) {
      case 'saving': return t('common.applying');
      case 'saved': return t('common.applied');
      case 'error': return t('common.tryAgain');
      default: return t('common.apply');
    }
  };

  const getSaveButtonClass = () => {
    const base = 'w-full px-4 py-2 rounded-lg transition-colors font-semibold ';
    switch (saveStatus) {
      case 'saving': return base + 'bg-muted text-muted-foreground cursor-not-allowed';
      case 'saved': return base + 'bg-green-600 dark:bg-green-700 text-white';
      case 'error': return base + 'bg-destructive text-destructive-foreground hover:bg-destructive/90';
      default: return base + 'bg-primary text-primary-foreground hover:bg-primary/90';
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <label className="block text-sm font-medium text-foreground mb-2">
          {t('settings.aiProvider.selectProvider')}
        </label>
        <Select
          value={selectedProvider}
          onValueChange={(value) => handleProviderChange(value as AiProviderType)}
        >
          <SelectTrigger>
            <SelectValue placeholder={t('settings.aiProvider.selectProvider')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="deepseek">DeepSeek</SelectItem>
            <SelectItem value="openai">OpenAI</SelectItem>
            <SelectItem value="anthropic">Anthropic</SelectItem>
            <SelectItem value="gemini">Gemini</SelectItem>
            <SelectItem value="ollama" disabled={!settingsStore.ollamaDetected}>
              Ollama {!settingsStore.ollamaDetected && `(${t('settings.aiProvider.notDetected')})`}
            </SelectItem>
          </SelectContent>
        </Select>
        <p className="mt-2 text-xs text-muted-foreground">
          {selectedProvider === 'deepseek'
            ? t('settings.aiProvider.deepseekHint')
            : selectedProvider === 'openai'
              ? t('settings.aiProvider.openaiHint')
              : t('settings.aiProvider.providerHint')}
        </p>
      </div>

      {(isCustomProvider || isLocalProvider) && (
        <div className="space-y-4">
          {isCustomProvider && (
            <>
              <div>
                <label className="block text-sm font-medium text-foreground mb-2">{t('settings.aiProvider.apiKey')}</label>
                <div className="relative">
                  <input
                    type="password"
                    value={apiKey}
                    onChange={(e) => {
                      setApiKey(e.target.value);
                      setApiKeyValid(null);
                      setAvailableModels([]);
                      setProbeResult(null);
                    }}
                    onBlur={() => {
                      if (apiKey.trim()) {
                        handleFetchModels();
                      }
                    }}
                    placeholder={t('settings.aiProvider.apiKeyPlaceholder')}
                    className="w-full px-4 py-2 pr-10 border border-input rounded-lg focus:border-ring focus:outline-none bg-background text-foreground"
                  />
                  {loadingModels && (
                    <Loader2 className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 animate-spin text-muted-foreground" />
                  )}
                </div>
                {apiKeyValid === true && (
                  <p className="mt-2 text-sm text-green-600 dark:text-green-400 flex items-center gap-1">
                    <Check className="h-4 w-4" />
                    <span>{t('settings.aiProvider.apiKeyValid', { count: availableModels.length })}</span>
                  </p>
                )}
                {apiKeyValid === false && (
                  <p className="mt-2 text-sm text-destructive flex items-center gap-1">
                    <X className="h-4 w-4" />
                    <span>{t('settings.aiProvider.apiKeyInvalid')}</span>
                  </p>
                )}
              </div>
            </>
          )}

          {isLocalProvider && (
            <div>
              <label className="block text-sm font-medium text-foreground mb-2">{t('settings.aiProvider.model')}</label>
              {settingsStore.ollamaModels.length > 0 ? (
                <Select
                  value={modelId}
                  onValueChange={(value) => {
                    setModelId(value);
                    setProbeResult(null);
                  }}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('settings.aiProvider.selectModel')} />
                  </SelectTrigger>
                  <SelectContent>
                    {settingsStore.ollamaModels.map((model) => (
                      <SelectItem key={model} value={model}>
                        {model}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              ) : (
                <div className="p-3 bg-muted rounded-lg border border-border text-sm text-muted-foreground">
                  {t('settings.aiProvider.noOllamaModels')}
                </div>
              )}
            </div>
          )}

          {isCustomProvider && (
            <div>
              <label className="block text-sm font-medium text-foreground mb-2">{t('settings.aiProvider.model')}</label>
              {availableModels.length > 0 ? (
                <div className="space-y-2">
                  <Select
                    value={modelId}
                    onValueChange={(value) => {
                    setModelId(value);
                    setProbeResult(null);
                  }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('settings.aiProvider.selectModel')} />
                    </SelectTrigger>
                    <SelectContent>
                      {availableModels.map((model) => (
                        <SelectItem key={model.id} value={model.id}>
                          <span className="flex items-center gap-2">
                            {model.name}
                            {RECOMMENDED_MODEL_IDS.has(model.id) && (
                              <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-semibold bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-400">
                                <Zap className="h-2.5 w-2.5" />
                                Recommended
                              </span>
                            )}
                          </span>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              ) : (
                <input
                  type="text"
                  value={modelId}
                  onChange={(e) => {
                    setModelId(e.target.value);
                    setProbeResult(null);
                  }}
                  placeholder={`e.g., ${getDefaultModel(selectedProvider)}`}
                  className="w-full px-4 py-2 border border-input rounded-lg focus:border-ring focus:outline-none bg-background text-foreground"
                />
              )}
            </div>
          )}

          <div className="space-y-2">
            <button
              type="button"
              onClick={handleProbeConnectivity}
              disabled={probing}
              className="w-full px-4 py-2 rounded-lg border border-input bg-background text-foreground hover:bg-muted transition-colors font-semibold inline-flex items-center justify-center gap-2 disabled:opacity-60"
            >
              {probing ? <Loader2 className="h-4 w-4 animate-spin" /> : <Activity className="h-4 w-4" />}
              {probing
                ? t('settings.aiProvider.connectivity.checking')
                : t('settings.aiProvider.connectivity.button')}
            </button>
            <p className="text-xs text-muted-foreground">
              {t('settings.aiProvider.connectivity.hint')}
            </p>
            {probeResult && (
              <div
                role="status"
                className={`rounded-lg border px-3 py-3 text-sm ${
                  probeResult.ok
                    ? 'border-green-600/40 bg-green-600/10 text-green-800 dark:text-green-300'
                    : 'border-destructive/40 bg-destructive/10 text-destructive'
                }`}
              >
                <p>{probeSummary(probeResult)}</p>
                {probeResult.replyPreview && (
                  <p className="mt-2 text-foreground">
                    <span className="font-medium">{t('settings.aiProvider.connectivity.reply')}: </span>
                    {probeResult.replyPreview}
                  </p>
                )}
                {probeResult.rawError && (
                  <div className="mt-2">
                    <p className="text-xs font-medium">{t('settings.aiProvider.connectivity.raw')}</p>
                    <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-all text-xs text-foreground/80">
                      {probeResult.rawError}
                    </pre>
                  </div>
                )}
              </div>
            )}
          </div>

          <button
            onClick={handleSaveConfiguration}
            disabled={
              saveStatus === 'saving' ||
              (isCustomProvider && (!apiKey.trim() || !modelId.trim())) ||
              (isLocalProvider && !modelId.trim())
            }
            className={getSaveButtonClass()}
          >
            {getSaveButtonText()}
          </button>
        </div>
      )}
    </div>
  );
});
