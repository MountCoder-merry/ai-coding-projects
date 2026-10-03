import { useMemo, useState } from "react";
import { Icon } from "../components/icons/Icon";
import { StatusPill } from "../components/StatusPill";
import { applyPlan } from "../services/runtime";
import { useAppStore } from "../store/appStore";

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function PreviewPage() {
  const {
    organizationPlan,
    setOrganizationPlan,
    isPlanning,
    setIsPlanning,
    planError,
    setPlanError,
    setLastApply,
    addTransaction,
    addActivity,
    setActiveView,
  } = useAppStore();
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set(organizationPlan?.operations.map((operation) => operation.id) ?? []));
  const [isApplying, setIsApplying] = useState(false);

  const selectedOperations = useMemo(
    () => organizationPlan?.operations.filter((operation) => selectedIds.has(operation.id)) ?? [],
    [organizationPlan, selectedIds],
  );

  if (!organizationPlan) {
    return (
      <div className="page-shell placeholder-page">
        <div className="placeholder-mark"><Icon name="scan" size={26} /></div>
        <div className="eyebrow">Preview / No plan</div>
        <h1>Scan a folder first.</h1>
        <p className="page-lede">A plan is always generated before any file operation can be applied.</p>
        <button className="secondary-button" onClick={() => setActiveView("organize")} type="button"><Icon name="folder" size={15} /> Back to organize</button>
      </div>
    );
  }

  function toggleOperation(id: string) {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }

  async function handleApply() {
    if (!organizationPlan || selectedOperations.length === 0 || isApplying) return;
    setIsApplying(true);
    setPlanError(null);
    try {
      const result = await applyPlan({ ...organizationPlan, operations: selectedOperations });
      setLastApply(result);
      addTransaction(result.transaction);
      addActivity({
        id: `apply-${Date.now()}`,
        title: "Organization applied",
        detail: `${result.completedCount} moved · ${result.failedCount} failed`,
        timestamp: "Just now",
        tone: result.failedCount > 0 ? "warning" : "success",
      });
      setOrganizationPlan(null);
      setActiveView("history");
    } catch (error) {
      setPlanError(error instanceof Error ? error.message : "The plan could not be applied.");
    } finally {
      setIsApplying(false);
    }
  }

  const newFolderCount = new Set(selectedOperations.map((operation) => operation.destination.split(/[\\/]/).slice(-2, -1)[0]).filter(Boolean)).size;

  return (
    <div className="page-shell">
      <header className="page-header compact-header">
        <div>
          <div className="eyebrow">Preview / Confirm plan</div>
          <h1>Make space, deliberately.</h1>
          <p className="page-lede">Review every destination before anything changes on disk.</p>
        </div>
        <StatusPill label="Nothing moved yet" tone="warning" />
      </header>

      {planError && <div className="notice-banner notice-banner--error"><div className="notice-icon"><Icon name="alert" size={16} /></div><div><strong>Plan action needs attention.</strong><span>{planError}</span></div><button className="subtle-button" onClick={() => setPlanError(null)} type="button">Dismiss</button></div>}

      <section className="preview-summary surface-card">
        <div><div className="card-kicker">Proposed changes</div><h2>{selectedOperations.length} files selected</h2></div>
        <div className="preview-metrics"><span><strong>{newFolderCount}</strong> new folders</span><span><strong>{formatBytes(selectedOperations.reduce((sum, operation) => sum + operation.sizeBytes, 0))}</strong> affected</span></div>
      </section>

      <section className="preview-tree surface-card">
        <div className="preview-tree-heading"><div><div className="card-kicker">Current → after organization</div><h2>Destination map</h2></div><span className="summary-caption">{organizationPlan.rootPath}</span></div>
        <div className="preview-operation-list">
          {organizationPlan.operations.map((operation) => {
            const selected = selectedIds.has(operation.id);
            return (
              <label className={`preview-operation ${selected ? "is-selected" : ""}`} key={operation.id}>
                <input checked={selected} onChange={() => toggleOperation(operation.id)} type="checkbox" />
                <span className="preview-check"><Icon name="check" size={12} /></span>
                <span className="preview-path"><strong>{operation.source.split(/[\\/]/).pop()}</strong><small>{operation.source} <Icon name="arrow-up-right" size={11} /> {operation.destination}</small></span>
                <span className="preview-reason">{operation.reason}</span>
              </label>
            );
          })}
          {organizationPlan.operations.length === 0 && <div className="empty-result"><Icon name="check" size={18} /><span>No moves are needed for this scan.</span></div>}
        </div>
      </section>

      <div className="page-bottom-actions">
        <button className="text-button" disabled={isPlanning || isApplying} onClick={() => setActiveView("organize")} type="button"><Icon name="chevron-right" size={14} style={{ transform: "rotate(180deg)" }} /> Back to scan</button>
        <button className="primary-button primary-button--large" disabled={selectedOperations.length === 0 || isApplying} onClick={() => void handleApply()} type="button"><Icon name="check" size={16} /> {isApplying ? "Applying…" : "Apply organization"}</button>
      </div>
    </div>
  );
}
