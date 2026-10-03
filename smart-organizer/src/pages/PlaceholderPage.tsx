import type { ViewId } from "../types/app";
import { Icon } from "../components/icons/Icon";
import { useAppStore } from "../store/appStore";

const pageCopy: Record<Exclude<ViewId, "home" | "organize" | "preview">, { eyebrow: string; title: string; body: string; icon: "sliders" | "clock" | "settings" }> = {
  rules: {
    eyebrow: "Rules / Coming next",
    title: "Give your files a language.",
    body: "Structured rules will let you describe intent without giving an AI direct access to your files.",
    icon: "sliders",
  },
  history: {
    eyebrow: "History / Coming next",
    title: "Every move leaves a trail.",
    body: "Transactions, partial failures, and reversible operations will live here — outside React state.",
    icon: "clock",
  },
  settings: {
    eyebrow: "Settings / Coming next",
    title: "Set your safety boundaries.",
    body: "Scan policy, conflict behavior, and local storage preferences will be configurable here.",
    icon: "settings",
  },
};

export function PlaceholderPage({ view }: { view: Exclude<ViewId, "home" | "organize" | "preview"> }) {
  const { setActiveView } = useAppStore();
  const content = pageCopy[view];

  return (
    <div className="page-shell placeholder-page">
      <div className="placeholder-mark"><Icon name={content.icon} size={26} /></div>
      <div className="eyebrow">{content.eyebrow}</div>
      <h1>{content.title}</h1>
      <p className="page-lede">{content.body}</p>
      <div className="placeholder-rule" />
      <div className="placeholder-note"><Icon name="shield" size={15} /><span>Phase 1 keeps this surface intentionally quiet while the file engine is built.</span></div>
      <button className="secondary-button" onClick={() => setActiveView("home")} type="button"><Icon name="home" size={15} /> Return home</button>
    </div>
  );
}
