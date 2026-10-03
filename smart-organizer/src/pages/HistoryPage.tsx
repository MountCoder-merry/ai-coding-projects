import { useState } from "react";
import { Icon } from "../components/icons/Icon";
import { StatusPill } from "../components/StatusPill";
import { undoTransaction } from "../services/runtime";
import { useAppStore } from "../store/appStore";
import type { OperationTransaction } from "../types/organization";

function transactionTone(transaction: OperationTransaction): "success" | "warning" | "quiet" {
  if (transaction.undoneAt) return "quiet";
  return transaction.failedCount > 0 ? "warning" : "success";
}

function transactionLabel(transaction: OperationTransaction) {
  if (transaction.undoneAt) return "Undone";
  if (transaction.failedCount > 0) return "Partially failed";
  return "Completed";
}

function formatDate(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

export function HistoryPage() {
  const { transactions, updateTransaction, addActivity, setActiveView } = useAppStore();
  const [busyId, setBusyId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleUndo(transaction: OperationTransaction) {
    if (busyId || transaction.undoneAt) return;
    setBusyId(transaction.id);
    setError(null);
    try {
      const result = await undoTransaction(transaction);
      updateTransaction(result.transaction);
      addActivity({
        id: `undo-${Date.now()}`,
        title: "Transaction undone",
        detail: `${result.completedCount} files restored · ${result.failedCount} failed`,
        timestamp: "Just now",
        tone: result.failedCount > 0 ? "warning" : "success",
      });
    } catch (undoError) {
      setError(undoError instanceof Error ? undoError.message : "The transaction could not be undone.");
    } finally {
      setBusyId(null);
    }
  }

  return (
    <div className="page-shell">
      <header className="page-header compact-header">
        <div>
          <div className="eyebrow">History / Local transactions</div>
          <h1>Every move leaves a trail.</h1>
          <p className="page-lede">Transactions stay local so every successful move can be reviewed and restored.</p>
        </div>
        <StatusPill label="Undo available" tone="success" />
      </header>

      {error && <div className="notice-banner notice-banner--error"><div className="notice-icon"><Icon name="alert" size={16} /></div><div><strong>Undo could not complete.</strong><span>{error}</span></div><button className="subtle-button" onClick={() => setError(null)} type="button">Dismiss</button></div>}

      {transactions.length === 0 ? (
        <section className="history-empty surface-card">
          <div className="placeholder-mark"><Icon name="clock" size={24} /></div>
          <div><div className="card-kicker">No transactions yet</div><h2>Run a plan to start your local history.</h2><p>Completed and partially failed operations will appear here with an Undo action.</p></div>
          <button className="secondary-button" onClick={() => setActiveView("organize")} type="button"><Icon name="folder" size={15} /> Organize a folder</button>
        </section>
      ) : (
        <section className="history-list surface-card">
          {transactions.map((transaction) => (
            <article className="history-row" key={transaction.id}>
              <div className="history-icon"><Icon name={transaction.undoneAt ? "undo" : "check"} size={16} /></div>
              <div className="history-copy"><strong>Organized folder</strong><span>{formatDate(transaction.createdAt)} · {transaction.rootPath}</span></div>
              <div className="history-count"><strong>{transaction.completedCount}</strong><span>moved</span></div>
              <StatusPill label={transactionLabel(transaction)} tone={transactionTone(transaction)} />
              <button className="small-button" disabled={Boolean(transaction.undoneAt) || busyId === transaction.id} onClick={() => void handleUndo(transaction)} type="button"><Icon name="undo" size={13} /> {busyId === transaction.id ? "Undoing…" : "Undo"}</button>
            </article>
          ))}
        </section>
      )}
    </div>
  );
}
