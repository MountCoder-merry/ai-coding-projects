import { useMemo, useState } from "react";
import { Icon } from "../components/icons/Icon";
import { StatusPill } from "../components/StatusPill";
import { generatePlan, scanFolder, pickDirectory } from "../services/runtime";
import { useAppStore } from "../store/appStore";
import type { ScannedFile } from "../types/app";

type CategoryCardLabel = "Images" | "Documents" | "Videos" | "Others";

const categories: Array<{ label: CategoryCardLabel; color: string }> = [
  { label: "Images", color: "mint" },
  { label: "Documents", color: "blue" },
  { label: "Videos", color: "violet" },
  { label: "Others", color: "gray" },
];

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatModified(file: ScannedFile) {
  if (!file.modifiedAt) return "—";
  const date = new Date(file.modifiedAt);
  return Number.isNaN(date.getTime()) ? "—" : date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

export function OrganizePage() {
  const {
    selectedFolder,
    setSelectedFolder,
    scanResult,
    isScanning,
    scanError,
    isPlanning,
    planError,
    setScanResult,
    setIsScanning,
    setScanError,
    setOrganizationPlan,
    setIsPlanning,
    setPlanError,
    addActivity,
    setActiveView,
  } = useAppStore();
  const [isPicking, setIsPicking] = useState(false);

  const categoryCounts = useMemo(() => {
    const counts = scanResult?.summary.byCategory;
    return {
      Images: counts?.Images ?? 0,
      Documents: counts?.Documents ?? 0,
      Videos: counts?.Videos ?? 0,
      Others: counts ? Object.entries(counts).filter(([name]) => !["Images", "Documents", "Videos"].includes(name)).reduce((sum, [, value]) => sum + value, 0) : 0,
    };
  }, [scanResult]);

  async function handlePickFolder() {
    setIsPicking(true);
    const folder = await pickDirectory();
    setIsPicking(false);
    if (folder) {
      setSelectedFolder(folder);
      setScanResult(null);
      setScanError(null);
      setOrganizationPlan(null);
      setPlanError(null);
      addActivity({ id: `folder-${Date.now()}`, title: "Folder selected", detail: folder, timestamp: "Just now", tone: "neutral" });
    }
  }

  async function handleGeneratePlan() {
    if (!selectedFolder || !scanResult || isPlanning) return;
    setIsPlanning(true);
    setPlanError(null);
    try {
      const plan = await generatePlan(selectedFolder, scanResult);
      setOrganizationPlan(plan);
      addActivity({ id: `plan-${Date.now()}`, title: "Plan generated", detail: `${plan.summary.operationCount} safe moves are ready to review`, timestamp: "Just now", tone: "success" });
      setActiveView("preview");
    } catch (error) {
      const message = error instanceof Error ? error.message : "The organizer could not generate a plan.";
      setPlanError(message);
      addActivity({ id: `plan-error-${Date.now()}`, title: "Plan needs attention", detail: message, timestamp: "Just now", tone: "warning" });
    } finally {
      setIsPlanning(false);
    }
  }

  async function handleScan() {
    if (!selectedFolder || isScanning) return;
    setIsScanning(true);
    setScanError(null);
    try {
      const result = await scanFolder(selectedFolder);
      setScanResult(result);
      addActivity({
        id: `scan-${Date.now()}`,
        title: "Scan complete",
        detail: `${result.summary.totalFiles} files mapped · ${formatBytes(result.summary.totalBytes)}`,
        timestamp: "Just now",
        tone: "success",
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : "The local scanner could not read this folder.";
      setScanError(message);
      addActivity({ id: `scan-error-${Date.now()}`, title: "Scan needs attention", detail: message, timestamp: "Just now", tone: "warning" });
    } finally {
      setIsScanning(false);
    }
  }

  const visibleItems = scanResult?.items.slice(0, 8) ?? [];
  const totalFiles = scanResult?.summary.totalFiles ?? 0;

  return (
    <div className="page-shell">
      <header className="page-header compact-header">
        <div>
          <div className="eyebrow">Organize / New plan</div>
          <h1>Choose a folder.</h1>
          <p className="page-lede">Start with a single folder. The MVP reads metadata from its first layer only.</p>
        </div>
        <StatusPill label="Plan first · always" tone="quiet" />
      </header>

      <section className="folder-picker-card surface-card">
        <div className="folder-picker-visual"><Icon name="folder" size={28} /></div>
        <div className="folder-picker-content">
          <div className="card-kicker">Current folder</div>
          <h2>{selectedFolder ?? "No folder selected"}</h2>
          <p>{selectedFolder ? "Ready for a safe, metadata-only scan." : "Pick a folder to begin."}</p>
        </div>
        <div className="folder-picker-actions">
          <button className="secondary-button" disabled={isPicking} onClick={handlePickFolder} type="button">
            <Icon name="folder" size={16} /> {isPicking ? "Opening…" : "Change folder"}
          </button>
          <button className="primary-button" disabled={!selectedFolder || isScanning} onClick={() => void handleScan()} type="button">
            <Icon name="scan" size={16} /> {isScanning ? "Scanning…" : "Scan"}
          </button>
        </div>
      </section>

      {scanError && (
        <div className="notice-banner notice-banner--error">
          <div className="notice-icon"><Icon name="alert" size={16} /></div>
          <div><strong>Scan could not complete.</strong><span>{scanError}</span></div>
          <button className="subtle-button" onClick={() => setScanError(null)} type="button">Dismiss</button>
        </div>
      )}

      {planError && (
        <div className="notice-banner notice-banner--error">
          <div className="notice-icon"><Icon name="alert" size={16} /></div>
          <div><strong>Plan could not be generated.</strong><span>{planError}</span></div>
          <button className="subtle-button" onClick={() => setPlanError(null)} type="button">Dismiss</button>
        </div>
      )}

      <section className="summary-section">
        <div className="section-heading-row">
          <div><div className="card-kicker">Scan summary</div><h2>{scanResult ? `${totalFiles} files mapped` : "Nothing mapped yet"}</h2></div>
          <span className="summary-caption">{scanResult ? `Read ${scanResult.summary.skippedEntries} protected entries` : "Awaiting first scan"}</span>
        </div>
        <div className="category-grid">
          {categories.map((category) => (
            <div className="category-card" key={category.label}>
              <span className={`category-swatch category-swatch--${category.color}`} />
              <span className="category-label">{category.label}</span>
              <strong>{categoryCounts[category.label === "Others" ? "Others" : category.label]}</strong>
              <span className="category-foot">files</span>
            </div>
          ))}
        </div>
      </section>

      {scanResult && (
        <section className="scan-results surface-card">
          <div className="section-heading-row">
            <div><div className="card-kicker">Read-only file map</div><h2>First layer preview</h2></div>
            <span className="summary-caption">{formatBytes(scanResult.summary.totalBytes)} total</span>
          </div>
          {visibleItems.length === 0 ? (
            <div className="empty-result"><Icon name="folder" size={18} /><span>No visible files found in this folder.</span></div>
          ) : (
            <div className="scan-table" role="table" aria-label="Scanned files">
              <div className="scan-table-row scan-table-head" role="row"><span>Name</span><span>Category</span><span>Size</span><span>Modified</span></div>
              {visibleItems.map((file) => (
                <div className="scan-table-row" role="row" key={file.id}>
                  <span className="scan-file-name" title={file.path}><Icon name="file" size={14} />{file.name}</span>
                  <span>{file.category}</span>
                  <span>{formatBytes(file.sizeBytes)}</span>
                  <span>{formatModified(file)}</span>
                </div>
              ))}
            </div>
          )}
          {scanResult.items.length > visibleItems.length && <div className="table-footnote">Showing {visibleItems.length} of {scanResult.items.length} visible files.</div>}
          <div className="scan-result-actions">
            <span className="muted-caption"><Icon name="shield" size={13} /> Preview is generated locally before any move</span>
            <button className="primary-button" disabled={isPlanning || scanResult.summary.totalFiles === 0} onClick={() => void handleGeneratePlan()} type="button">
              <Icon name="spark" size={15} /> {isPlanning ? "Generating…" : "Generate plan"}
            </button>
          </div>
        </section>
      )}

      <section className="flow-strip surface-card">
        <div className="flow-step flow-step--active"><span>01</span><strong>Choose</strong><small>Pick a folder</small></div>
        <Icon name="chevron-right" size={16} className="flow-arrow" />
        <div className={`flow-step${scanResult ? " flow-step--active" : ""}`}><span>02</span><strong>Scan</strong><small>Read metadata only</small></div>
        <Icon name="chevron-right" size={16} className="flow-arrow" />
        <div className="flow-step"><span>03</span><strong>Preview</strong><small>Approve every move</small></div>
        <Icon name="chevron-right" size={16} className="flow-arrow" />
        <div className="flow-step"><span>04</span><strong>Undo</strong><small>Restore when needed</small></div>
      </section>

      <div className="page-bottom-actions">
        <button className="text-button" onClick={() => setActiveView("home")} type="button">Back home</button>
        <span className="muted-caption"><Icon name="shield" size={13} /> No file is changed by scanning</span>
      </div>
    </div>
  );
}
