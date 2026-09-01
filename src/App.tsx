import { useEffect } from "react";
import { Routes, Route } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Home } from "./pages/Home";
import { History } from "./pages/History";
import { Vocab } from "./pages/Vocab";
import ExplorePage from "./pages/ExplorePage";
import { Settings } from "./pages/Settings";
import { RootStore, RootStoreContext } from "./stores/RootStore";
import { AppLayout } from "./components/layout/AppLayout";
import { TitleBar } from "./components/layout/TitleBar";
import { Toaster } from "./components/ui/sonner";
import { api } from "./services/api";
import { LANGUAGE_MAP } from "./i18n";
import { useGlobalLookup } from "./hooks/useGlobalLookup";
import { useUpdateChecker } from "./hooks/useUpdateChecker";
import { UpdateDialog } from "./components/UpdateDialog";

const rootStore = new RootStore();

function App() {
  const { i18n } = useTranslation();

  useEffect(() => {
    const initLanguage = async () => {
      try {
        const settings = await api.getSettings();
        if (settings.uiLanguage) {
          const code = LANGUAGE_MAP[settings.uiLanguage] || 'en';
          await i18n.changeLanguage(code);
        }
      } catch (error) {
        console.error('Failed to initialize language:', error);
      }
    };
    
    initLanguage();
  }, [i18n]);

  useEffect(() => {
    document.documentElement.dir = i18n.dir();
    document.documentElement.lang = i18n.language;
  }, [i18n.language, i18n]);

  useEffect(() => {
    const showWindow = async () => {
      try {
        const appWindow = getCurrentWindow();
        await appWindow.show();
        await appWindow.setFocus();
      } catch (error) {
        console.error('Failed to show window:', error);
      }
    };
    
    const timer = setTimeout(showWindow, 300);
    return () => clearTimeout(timer);
  }, []);

  return (
    <RootStoreContext.Provider value={rootStore}>
      <GlobalLookupBridge />
      <UpdateBridge />
      <TitleBar />
      <AppLayout>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/explore" element={<ExplorePage />} />
          <Route path="/history" element={<History />} />
          <Route path="/vocab" element={<Vocab />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </AppLayout>
      <Toaster />
    </RootStoreContext.Provider>
  );
}

function GlobalLookupBridge() {
  useGlobalLookup();
  return null;
}

function UpdateBridge() {
  const {
    status,
    currentVersion,
    newVersion,
    downloadProgress,
    error,
    downloadAndInstall,
    restartApp,
    dismiss,
  } = useUpdateChecker({ autoCheck: true });

  // Only show dialog for real updates, not auto-check errors
  const showDialog = status === 'available' || status === 'downloading' || status === 'ready';

  return (
    <UpdateDialog
      open={showDialog}
      status={status}
      currentVersion={currentVersion}
      newVersion={newVersion}
      downloadProgress={downloadProgress}
      error={error}
      onDownload={downloadAndInstall}
      onRestart={restartApp}
      onDismiss={dismiss}
    />
  );
}

export default App;
