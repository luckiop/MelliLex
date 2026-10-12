import { useNavigate } from 'react-router-dom';
import { observer } from 'mobx-react-lite';
import { SearchBar } from './SearchBar';
import { NavigationTabs, MobileNavigationTabs } from './NavigationTabs';
import { Button } from '../ui/button';
import { ArrowLeft, Sun, Moon, Settings as SettingsIcon } from 'lucide-react';
import { useTheme } from '../../services/ThemeService';
import { useTranslation } from 'react-i18next';
import { useStores } from '../../stores/RootStore';

export const AppHeader = observer(function AppHeader() {
  const navigate = useNavigate();
  const { resolvedTheme, toggleTheme } = useTheme();
  const { t } = useTranslation();
  const {
    progressiveWordStore,
    phraseStore,
    zhEnStore,
    translateStore,
    searchCoordinator,
    exploreStore,
    lastPageStore,
  } = useStores();

  const canResetHome =
    Boolean(progressiveWordStore.currentWord) ||
    Boolean(progressiveWordStore.headerSection) ||
    progressiveWordStore.isLoading ||
    progressiveWordStore.hasError ||
    Boolean(phraseStore.currentPhrase) ||
    Boolean(phraseStore.overviewSection) ||
    phraseStore.isLoading ||
    phraseStore.hasError ||
    Boolean(zhEnStore.currentQuery) ||
    Boolean(zhEnStore.result) ||
    zhEnStore.isLoading ||
    zhEnStore.hasError ||
    Boolean(translateStore.currentSource) ||
    Boolean(translateStore.result) ||
    translateStore.isLoading ||
    translateStore.hasError;

  const handleResetHome = () => {
    searchCoordinator.resetToHome();
    exploreStore.resetAll();
    lastPageStore.setLastPage('/');
    navigate('/');
  };

  return (
    <>
      <div
        className="fixed left-0 right-0 z-50 overflow-visible border-b chrome-panel chrome-panel--top"
        style={{ top: 'var(--chrome-titlebar-height)', height: 'var(--chrome-header-height)' }}
      >
        <div className="max-w-3xl mx-auto px-6 h-full flex items-center">
          <div className="flex items-center gap-3 w-full">
            {canResetHome ? (
              <Button
                variant="ghost"
                size="icon"
                onClick={handleResetHome}
                title={t('navigation.backHome')}
                aria-label={t('navigation.backHome')}
              >
                <ArrowLeft className="h-5 w-5" />
              </Button>
            ) : null}
            <div className="flex-1">
              <SearchBar />
            </div>

            <div className="flex items-center gap-3 ml-auto">
              <div className="hidden md:block">
                <NavigationTabs />
              </div>

              <Button
                variant="ghost"
                size="icon"
                onClick={toggleTheme}
                className="hidden md:flex"
                title={resolvedTheme === 'dark' ? t('navigation.lightMode') : t('navigation.darkMode')}
              >
                {resolvedTheme === 'dark' ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
              </Button>

              <Button
                variant="ghost"
                size="icon"
                onClick={() => navigate('/settings')}
                className="hidden md:flex"
                title={t('common.settings')}
              >
                <SettingsIcon className="h-5 w-5" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <MobileNavigationTabs />
    </>
  );
});
