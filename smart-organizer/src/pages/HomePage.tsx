import { useAppStore } from "../store/appStore";
import { getDesktopFolder } from "../services/runtime";
import { Icon } from "../components/icons/Icon";
import { StatusPill } from "../components/StatusPill";

export function HomePage() {
  const { setActiveView, setSelectedFolder, recentActivity, selectedFolder, scanResult } = useAppStore();

  async function openDesktopOrganizer() {
    const desktop = await getDesktopFolder();
    if (desktop) setSelectedFolder(desktop);
    setActiveView("organize");
  }

  return (
    <div className="page-shell home-page">
      <header className="page-header home-header">
        <div>
          <div className="eyebrow">A calmer file system</div>
          <h1>Make space for<br /><span>what matters.</span></h1>
          <p className="page-lede">Smart Organizer turns a messy folder into a clear, reversible plan. Nothing moves until you say so.</p>
        </div>
        <div className="orbit-badge" aria-hidden="true">
          <div className="orbit-ring orbit-ring--one" />
          <div className="orbit-ring orbit-ring--two" />
          <div className="orbit-core"><Icon name="spark" size={25} /></div>
          <span className="orbit-caption">PLAN<br />FIRST</span>
        </div>
      </header>

      <section className="hero-actions" aria-label="Start organizing">
        <button className="primary-button primary-button--large" onClick={() => setActiveView("organize")} type="button">
          <span className="button-icon"><Icon name="folder" size={18} /></span>
          Organize a folder
          <Icon name="arrow-up-right" size={16} />
        </button>
        <button className="text-button" onClick={() => setActiveView("rules")} type="button">
          Explore rules <Icon name="chevron-right" size={15} />
        </button>
      </section>

      <div className="home-grid">
        <section className="surface-card desktop-card">
          <div className="card-heading-row">
            <div>
              <div className="card-kicker">Quick start</div>
              <h2>Your Desktop</h2>
            </div>
            <StatusPill label={scanResult ? "Scan ready" : "Ready to scan"} tone="success" />
          </div>
          <div className="path-row">
            <div className="path-icon"><Icon name="folder" size={17} /></div>
            <span>{selectedFolder ?? "Choose a folder to get started"}</span>
            <Icon name="chevron-right" size={15} />
          </div>
          <div className="card-divider" />
          <div className="desktop-card-footer">
            <div className="metric"><strong>{scanResult?.summary.totalFiles ?? 0}</strong><span>files mapped</span></div>
            <button className="small-button" onClick={() => void openDesktopOrganizer()} type="button">{scanResult ? "Review scan" : "Scan Desktop"} <Icon name="scan" size={14} /></button>
          </div>
        </section>

        <section className="surface-card safety-card">
          <div className="safety-icon"><Icon name="shield" size={18} /></div>
          <div>
            <div className="card-kicker">Safety promise</div>
            <h3>Preview every move.</h3>
            <p>Safe names, no silent overwrites, and one-click undo for every completed plan.</p>
          </div>
        </section>
      </div>

      <section className="activity-section">
        <div className="section-heading-row">
          <div>
            <div className="card-kicker">A quiet log</div>
            <h2>Recent activity</h2>
          </div>
          <button className="subtle-button" onClick={() => setActiveView("history")} type="button">View history <Icon name="arrow-up-right" size={14} /></button>
        </div>
        <div className="activity-list">
          {recentActivity.map((activity) => (
            <div className="activity-row" key={activity.id}>
              <div className={`activity-bullet activity-bullet--${activity.tone}`}><Icon name={activity.tone === "success" ? "check" : "spark"} size={13} /></div>
              <div className="activity-copy"><strong>{activity.title}</strong><span>{activity.detail}</span></div>
              <time>{activity.timestamp}</time>
              <Icon name="more" size={17} className="muted-icon" />
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
