import type { ViewId } from "../../types/app";
import { Icon, type IconName } from "../icons/Icon";

interface SidebarProps {
  activeView: ViewId;
  onNavigate: (view: ViewId) => void;
  engineStatus: "checking" | "ready" | "browser" | "offline";
}

const primaryItems: Array<{ id: ViewId; label: string; icon: IconName }> = [
  { id: "home", label: "Home", icon: "home" },
  { id: "organize", label: "Organize", icon: "folder" },
  { id: "rules", label: "Rules", icon: "sliders" },
  { id: "history", label: "History", icon: "clock" },
];

export function Sidebar({ activeView, onNavigate, engineStatus }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand-lockup">
        <div className="brand-mark" aria-hidden="true">
          <Icon name="spark" size={19} />
        </div>
        <div>
          <div className="brand-name">Smart Organizer</div>
          <div className="brand-version">Private workspace</div>
        </div>
      </div>

      <div className="sidebar-section-label">Workspace</div>
      <nav className="sidebar-nav" aria-label="Main navigation">
        {primaryItems.map((item) => (
          <button
            className={`nav-item ${(activeView === item.id || (item.id === "organize" && activeView === "preview")) ? "is-active" : ""}`}
            key={item.id}
            onClick={() => onNavigate(item.id)}
            type="button"
          >
            <Icon name={item.icon} size={17} />
            <span>{item.label}</span>
            {item.id === "organize" && <span className="nav-kicker">01</span>}
          </button>
        ))}
      </nav>

      <div className="sidebar-spacer" />

      <div className="engine-card">
        <div className="engine-card-topline">
          <span className={`status-dot status-dot--${engineStatus}`} />
          <span>{engineStatus === "ready" ? "Rust engine ready" : engineStatus === "browser" ? "Preview mode" : "Checking engine"}</span>
        </div>
        <p>Plans are created before anything moves.</p>
      </div>

      <button
        className={`nav-item settings-item ${activeView === "settings" ? "is-active" : ""}`}
        onClick={() => onNavigate("settings")}
        type="button"
      >
        <Icon name="settings" size={17} />
        <span>Settings</span>
      </button>

      <div className="sidebar-footer">
        <span className="sidebar-footer-dot" />
        <span>Local only</span>
        <span className="sidebar-footer-separator">·</span>
        <span>v0.1</span>
      </div>
    </aside>
  );
}
