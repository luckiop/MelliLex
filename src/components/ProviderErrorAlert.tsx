import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Alert, AlertTitle, AlertDescription } from './ui/alert';
import { Button } from './ui/button';
import { Settings, AlertTriangle, RefreshCw } from 'lucide-react';
import { isProviderConfigError, getProviderConfigErrorMessage, isModelStallError } from '../utils/errorHandler';

interface Props {
  error: string | null;
  rawError?: unknown;
  onRetry?: () => void;
}

/**
 * Alert component that detects provider configuration errors
 * and provides a link to settings for the user to configure their API.
 */
export const ProviderErrorAlert = ({ error, rawError, onRetry }: Props) => {
  const navigate = useNavigate();
  const { t } = useTranslation();
  
  if (!error) return null;
  
  const stall = isModelStallError(rawError ?? error);
  const isConfigError = rawError 
    ? isProviderConfigError(rawError) 
    : isProviderConfigError({ message: error });
  
  const displayMessage = stall
    ? t('errors.modelStalled')
    : isConfigError
    ? getProviderConfigErrorMessage(rawError ?? { message: error })
    : error;
  const rawText = stall
    ? (typeof rawError === 'string' ? rawError : error)
    : null;

  const handleGoToSettings = () => {
    navigate('/settings');
  };

  return (
    <Alert variant="destructive" className="mb-6">
      <div className="flex gap-3">
        <AlertTriangle className="h-4 w-4 mt-0.5 flex-shrink-0" />
        <div className="flex-1">
          <AlertTitle>
            {isConfigError ? t('errors.providerConfigurationRequired') : t('common.error')}
          </AlertTitle>
          <AlertDescription className="mt-2">
            <p className="mb-3">{displayMessage}</p>
            {rawText && (
              <div className="mb-3">
                <p className="text-xs font-medium">{t('errors.rawError')}</p>
                <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-all text-xs opacity-80">
                  {rawText}
                </pre>
              </div>
            )}
            <div className="flex gap-2">
              {(isConfigError || stall) && (
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={handleGoToSettings}
                  className="gap-2"
                >
                  <Settings className="h-4 w-4" />
                  {t('common.settings')}
                </Button>
              )}
              {onRetry && (
                <Button 
                  variant="ghost" 
                  size="sm" 
                  onClick={onRetry}
                  className="gap-2"
                >
                  <RefreshCw className="h-4 w-4" />
                  {t('common.tryAgain')}
                </Button>
              )}
            </div>
          </AlertDescription>
        </div>
      </div>
    </Alert>
  );
};
