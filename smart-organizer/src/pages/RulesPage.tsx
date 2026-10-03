import { useState } from "react";
import { Icon } from "../components/icons/Icon";
import { StatusPill } from "../components/StatusPill";
import { parseRuleMock, saveRule, validateRule } from "../services/runtime";
import { useAppStore } from "../store/appStore";
import type { RuleCondition, RuleField, RuleOperator, StructuredRule } from "../types/rules";

const fieldOptions: Array<{ value: RuleField; label: string }> = [
  { value: "filename", label: "Filename" },
  { value: "extension", label: "Extension" },
  { value: "category", label: "Category" },
  { value: "size", label: "Size (bytes)" },
  { value: "createdAt", label: "Created date" },
  { value: "modifiedAt", label: "Modified date" },
];

const operatorOptions: Array<{ value: RuleOperator; label: string }> = [
  { value: "equals", label: "equals" },
  { value: "contains", label: "contains" },
  { value: "startsWith", label: "starts with" },
  { value: "endsWith", label: "ends with" },
  { value: "greaterThan", label: "greater than" },
  { value: "lessThan", label: "less than" },
  { value: "olderThan", label: "older than (days)" },
  { value: "newerThan", label: "newer than (days)" },
];

function createDraft(): StructuredRule {
  return {
    id: `rule-${Date.now()}`,
    name: "New folder rule",
    conditions: [{ field: "filename", operator: "contains", value: "Screenshot" }],
    action: { type: "move", destination: "Screenshots" },
    enabled: true,
  };
}

export function RulesPage() {
  const { rules, setRules, addActivity } = useAppStore();
  const [draft, setDraft] = useState<StructuredRule>(() => createDraft());
  const [naturalText, setNaturalText] = useState("");
  const [isParsing, setIsParsing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [messageTone, setMessageTone] = useState<"success" | "error">("success");

  function updateCondition(change: Partial<RuleCondition>) {
    setDraft((current) => ({ ...current, conditions: [{ ...current.conditions[0], ...change }] }));
  }

  async function handleParse() {
    if (!naturalText.trim() || isParsing) return;
    setIsParsing(true);
    setMessage(null);
    try {
      const parsed = await parseRuleMock(naturalText);
      setDraft(parsed);
      setMessage("Parsed into a structured rule draft. Review it before saving.");
      setMessageTone("success");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "The parser could not understand that request.");
      setMessageTone("error");
    } finally {
      setIsParsing(false);
    }
  }

  async function handleSave() {
    if (isSaving) return;
    setIsSaving(true);
    setMessage(null);
    try {
      await validateRule(draft);
      const savedRules = await saveRule(draft);
      setRules(savedRules);
      addActivity({ id: `rule-${Date.now()}`, title: "Rule saved", detail: `${draft.name} · ${draft.action.destination}`, timestamp: "Just now", tone: "success" });
      setMessage("Rule saved locally. The deterministic engine will apply it during the next plan.");
      setMessageTone("success");
      setDraft(createDraft());
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "The rule is not valid.");
      setMessageTone("error");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="page-shell">
      <header className="page-header compact-header">
        <div>
          <div className="eyebrow">Rules / Structured intent</div>
          <h1>Give your files a language.</h1>
          <p className="page-lede">Describe intent, review the structured rule, and let the deterministic engine decide what is safe.</p>
        </div>
        <StatusPill label="AI suggests · engine decides" tone="quiet" />
      </header>

      {message && <div className={`notice-banner ${messageTone === "error" ? "notice-banner--error" : "notice-banner--rule-success"}`}><div className="notice-icon"><Icon name={messageTone === "error" ? "alert" : "check"} size={16} /></div><div><strong>{messageTone === "error" ? "Rule needs attention." : "Rule workspace updated."}</strong><span>{message}</span></div><button className="subtle-button" onClick={() => setMessage(null)} type="button">Dismiss</button></div>}

      <section className="rule-parser-card surface-card">
        <div className="card-kicker">Natural language interface · mock parser</div>
        <h2>Start with a sentence.</h2>
        <p>Try “以后所有截图都放到 Screenshots 文件夹” or “所有超过 1GB 的视频移动到 Large Videos”.</p>
        <div className="rule-parser-row">
          <textarea value={naturalText} onChange={(event) => setNaturalText(event.target.value)} placeholder="Describe a safe organizing rule…" rows={2} />
          <button className="primary-button" disabled={!naturalText.trim() || isParsing} onClick={() => void handleParse()} type="button"><Icon name="spark" size={15} /> {isParsing ? "Parsing…" : "Parse draft"}</button>
        </div>
      </section>

      <section className="rule-editor-grid">
        <div className="rule-editor-card surface-card">
          <div className="section-heading-row"><div><div className="card-kicker">Manual editor</div><h2>Review the rule</h2></div><span className="summary-caption">No file access</span></div>
          <label className="field-label">Rule name<input value={draft.name} onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))} /></label>
          <div className="condition-heading"><span className="field-label">Condition</span><span className="summary-caption">all must match</span></div>
          <div className="condition-row">
            <select value={draft.conditions[0]?.field} onChange={(event) => updateCondition({ field: event.target.value as RuleField })}>{fieldOptions.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}</select>
            <select value={draft.conditions[0]?.operator} onChange={(event) => updateCondition({ operator: event.target.value as RuleOperator })}>{operatorOptions.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}</select>
            <input value={draft.conditions[0]?.value ?? ""} onChange={(event) => updateCondition({ value: event.target.value })} placeholder="Value" />
          </div>
          <label className="field-label">Action<select value={draft.action.type} onChange={(event) => setDraft((current) => ({ ...current, action: { ...current.action, type: event.target.value as "move" | "rename" } }))}><option value="move">Move to folder</option><option value="rename">Rename with pattern</option></select></label>
          <label className="field-label">Destination<input value={draft.action.destination} onChange={(event) => setDraft((current) => ({ ...current, action: { ...current.action, destination: event.target.value } }))} placeholder="Screenshots" /></label>
          <button className="primary-button" disabled={isSaving} onClick={() => void handleSave()} type="button"><Icon name="check" size={15} /> {isSaving ? "Validating…" : "Save rule"}</button>
        </div>

        <div className="rule-json-card surface-card">
          <div className="section-heading-row"><div><div className="card-kicker">Structured output</div><h2>What the engine receives</h2></div><Icon name="shield" size={18} /></div>
          <pre>{JSON.stringify(draft, null, 2)}</pre>
          <div className="rule-safety-note"><Icon name="shield" size={14} /><span>Rules can suggest destinations. Only the plan and filesystem engine can move files.</span></div>
        </div>
      </section>

      <section className="saved-rules-section">
        <div className="section-heading-row"><div><div className="card-kicker">Local rule set</div><h2>{rules.length ? `${rules.length} saved rules` : "No saved rules yet"}</h2></div></div>
        {rules.length > 0 && <div className="saved-rules-list">{rules.map((rule) => <div className="saved-rule-row" key={rule.id}><span className="saved-rule-dot" /><div><strong>{rule.name}</strong><span>{rule.conditions[0]?.field} {rule.conditions[0]?.operator} “{rule.conditions[0]?.value}” → {rule.action.destination}</span></div><StatusPill label={rule.enabled ? "Enabled" : "Paused"} tone={rule.enabled ? "success" : "quiet"} /></div>)}</div>}
      </section>
    </div>
  );
}
