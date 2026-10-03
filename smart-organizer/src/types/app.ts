export type ViewId = "home" | "organize" | "preview" | "rules" | "history" | "settings";

export type EngineStatus = "checking" | "ready" | "browser" | "offline";

export interface AppInfo {
  name: string;
  version: string;
  engineStatus: string;
}

export interface RecentActivity {
  id: string;
  title: string;
  detail: string;
  timestamp: string;
  tone: "success" | "neutral" | "warning";
}

export type FileCategory =
  | "Images"
  | "Videos"
  | "Documents"
  | "Archives"
  | "Audio"
  | "Code"
  | "Applications"
  | "Other";

export interface ScannedFile {
  id: string;
  name: string;
  path: string;
  extension: string | null;
  category: FileCategory;
  sizeBytes: number;
  createdAt: string | null;
  modifiedAt: string | null;
  isDirectory: boolean;
}

export interface ScanSummary {
  totalFiles: number;
  totalBytes: number;
  skippedEntries: number;
  byCategory: Record<FileCategory, number>;
}

export interface ScanResult {
  root: string;
  scannedAt: string;
  items: ScannedFile[];
  summary: ScanSummary;
}
