import { useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Button } from '../ui/button';
import { Sun, Moon, Settings as SettingsIcon } from 'lucide-react';
import { useTheme } from '../../services/ThemeService';

export function NavigationTabs() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();

  const isActive = (path: string) => location.pathname === path;

  const tabs = [
    { path: '/', label: t('navigation.home') },
    { path: '/history', label: t('navigation.history') },
    { path: '/vocab', label: t('navigation.vocab') },
  ];

  return (
    <nav className="flex items-center gap-1">
      {tabs.map(({ path, label }) => (
        <Button
          key={path}
          variant="ghost"
          size="sm"
          onClick={() => navigate(path)}
          className={`relative ${
            isActive(path)
              ? 'bg-accent text-accent-foreground'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          {label}
        </Button>
      ))}
    </nav>
  );
}

export function MobileNavigationTabs() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const { resolvedTheme, toggleTheme } = useTheme();

  const isActive = (path: string) => location.pathname === path;

  const tabs = [
    { path: '/', label: t('navigation.home') },
    { path: '/history', label: t('navigation.history') },
    { path: '/vocab', label: t('navigation.vocab') },
  ];

  return (
    <div
      className="md:hidden sticky z-40 bg-card border-b border-border"
      style={{ top: 'calc(var(--chrome-titlebar-height) + var(--chrome-header-height))' }}
    >
      <div className="flex overflow-x-auto gap-1 p-2">
        {tabs.map(({ path, label }) => (
          <Button
            key={path}
            variant="ghost"
            size="sm"
            onClick={() => navigate(path)}
            className={`flex-shrink-0 ${
              isActive(path)
                ? 'bg-accent text-accent-foreground'
                : 'text-muted-foreground'
            }`}
          >
            {label}
          </Button>
        ))}
        
        <Button
          variant="ghost"
          size="icon"
          onClick={toggleTheme}
          title={resolvedTheme === 'dark' ? t('navigation.lightMode') : t('navigation.darkMode')}
        >
          {resolvedTheme === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
        </Button>
        
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate('/settings')}
          title={t('common.settings')}
        >
          <SettingsIcon className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
