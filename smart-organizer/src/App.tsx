import { useEffect } from "react";
import { Sidebar } from "./components/layout/Sidebar";
import { Icon } from "./components/icons/Icon";
import { HomePage } from "./pages/HomePage";
import { HistoryPage } from "./pages/HistoryPage";
import { OrganizePage } from "./pages/OrganizePage";
import { PreviewPage } from "./pages/PreviewPage";
import { RulesPage } from "./pages/RulesPage";
import { PlaceholderPage } from "./pages/PlaceholderPage";
import { checkEngine, loadAppInfo, loadHistory, loadRules } from "./services/runtime";
import { useAppStore } from "./store/appStore";
import type { ViewId } from "./types/app";

export default function App() {
  const { activeView, engineStatus, setActiveView, setEngineStatus, setAppInfo, setTransactions, setRules } = useAppStore();

  useEffect(() => {
    let cancelled = false;
    void Promise.all([checkEngine(), loadAppInfo(), loadHistory().catch(() => []), loadRules().catch(() => [])]).then(([status, info, history, savedRules]) => {
      if (cancelled) return;
      setEngineStatus(status);
      setAppInfo(info);
      setTransactions(history);
      setRules(savedRules);
    });
    return () => { cancelled = true; };
  }, [setAppInfo, setEngineStatus, setRules, setTransactions]);

  function renderPage(view: ViewId) {
    if (view === "home") return <HomePage />;
    if (view === "organize") return <OrganizePage />;
    if (view === "preview") return <PreviewPage />;
    if (view === "history") return <HistoryPage />;
    if (view === "rules") return <RulesPage />;
    return <PlaceholderPage view={view} />;
  }

  return (
    <div className="app-shell">
      <Sidebar activeView={activeView} onNavigate={setActiveView} engineStatus={engineStatus} />
      <main className="main-column">
        <div className="topbar">
          <div className="breadcrumb"><span>Workspace</span><Icon name="chevron-right" size={13} /><strong>{activeView[0].toUpperCase() + activeView.slice(1)}</strong></div>
          <div className="topbar-actions">
            <span className="local-badge"><span /> Local workspace</span>
            <button className="icon-button" aria-label="Open settings" onClick={() => setActiveView("settings")} type="button"><Icon name="settings" size={16} /></button>
          </div>
        </div>
        <div className="page-scroll">{renderPage(activeView)}</div>
      </main>
    </div>
  );
}
