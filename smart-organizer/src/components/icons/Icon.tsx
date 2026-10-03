import type { ReactNode, SVGProps } from "react";

export type IconName =
  | "spark"
  | "home"
  | "folder"
  | "sliders"
  | "clock"
  | "settings"
  | "arrow-up-right"
  | "scan"
  | "chevron-right"
  | "check"
  | "plus"
  | "file"
  | "shield"
  | "undo"
  | "more"
  | "sun"
  | "moon"
  | "alert";

const paths: Record<IconName, ReactNode> = {
  spark: (
    <>
      <path d="m12 2 1.35 5.2L18 9l-4.65 1.8L12 16l-1.35-5.2L6 9l4.65-1.8L12 2Z" />
      <path d="m19 15 .7 2.3L22 18l-2.3.7L19 21l-.7-2.3L16 18l2.3-.7L19 15Z" />
    </>
  ),
  home: <path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V10Zm6 11v-6h6v6" />,
  folder: <path d="M3 6.5A1.5 1.5 0 0 1 4.5 5h5l2 2h8A1.5 1.5 0 0 1 21 8.5v9A1.5 1.5 0 0 1 19.5 19h-15A1.5 1.5 0 0 1 3 17.5v-11Z" />,
  sliders: (
    <>
      <path d="M4 6h16M4 12h16M4 18h16" />
      <path d="M8 4v4M16 10v4M10 16v4" />
    </>
  ),
  clock: <path d="M12 7v5l3 2m6-2a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />,
  settings: (
    <>
      <path d="M12 15.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Z" />
      <path d="m19.4 15 .1.1a1.8 1.8 0 0 1-2.5 2.5l-.1-.1a1.8 1.8 0 0 0-3.1 1.3v.2a1.8 1.8 0 0 1-3.6 0v-.2a1.8 1.8 0 0 0-3.1-1.3l-.1.1a1.8 1.8 0 1 1-2.5-2.5l.1-.1a1.8 1.8 0 0 0-1.3-3.1h-.2a1.8 1.8 0 0 1 0-3.6h.2a1.8 1.8 0 0 0 1.3-3.1l-.1-.1A1.8 1.8 0 1 1 7 2.7l.1.1a1.8 1.8 0 0 0 3.1-1.3v-.2a1.8 1.8 0 0 1 3.6 0v.2a1.8 1.8 0 0 0 3.1 1.3l.1-.1a1.8 1.8 0 1 1 2.5 2.5l-.1.1a1.8 1.8 0 0 0 1.3 3.1h.2a1.8 1.8 0 0 1 0 3.6h-.2a1.8 1.8 0 0 0-1.3 3Z" />
    </>
  ),
  "arrow-up-right": <path d="M7 17 17 7m0 0H9m8 0v8" />,
  scan: (
    <>
      <path d="M4 8V5a1 1 0 0 1 1-1h3M16 4h3a1 1 0 0 1 1 1v3M20 16v3a1 1 0 0 1-1 1h-3M8 20H5a1 1 0 0 1-1-1v-3" />
      <path d="M7 12h10" />
    </>
  ),
  "chevron-right": <path d="m9 5 7 7-7 7" />,
  check: <path d="m5 12 4 4L19 6" />,
  plus: <path d="M12 5v14M5 12h14" />,
  file: <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6Z" />,
  shield: <path d="M12 3 20 6v5c0 5-3.2 8.5-8 10-4.8-1.5-8-5-8-10V6l8-3Zm-3 9 2 2 4-4" />,
  undo: <path d="M9 14 4 9l5-5m-5 5h10a6 6 0 0 1 6 6v1" />,
  more: <path d="M5 12h.01M12 12h.01M19 12h.01" />,
  sun: <circle cx="12" cy="12" r="4" />,
  moon: <path d="M20 15.5A8.5 8.5 0 0 1 8.5 4 8.5 8.5 0 1 0 20 15.5Z" />,
  alert: (
    <>
      <path d="M12 4 21 20H3L12 4Z" />
      <path d="M12 9v5m0 3h.01" />
    </>
  ),
};

export function Icon({ name, size = 18, ...props }: { name: IconName; size?: number } & SVGProps<SVGSVGElement>) {
  return (
    <svg
      aria-hidden="true"
      fill="none"
      height={size}
      viewBox="0 0 24 24"
      width={size}
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.7"
      {...props}
    >
      {paths[name]}
    </svg>
  );
}
