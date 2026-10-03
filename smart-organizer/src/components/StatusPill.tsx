import { Icon } from "./icons/Icon";

export function StatusPill({ label, tone = "quiet" }: { label: string; tone?: "quiet" | "success" | "warning" }) {
  return (
    <span className={`status-pill status-pill--${tone}`}>
      <Icon name={tone === "success" ? "check" : tone === "warning" ? "shield" : "spark"} size={13} />
      {label}
    </span>
  );
}
