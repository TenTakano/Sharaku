export type TimerHandle = ReturnType<typeof setTimeout> | null;

export type ViewKind =
  | "library"
  | "viewer"
  | "settings"
  | "import"
  | "bulk-import"
  | "add-library"
  | "app-settings";

export interface WorkSummary {
  id: number;
  title: string;
  workType: string;
  pageCount: number;
  createdAt: string;
}

export interface WorkDetail {
  id: number;
  title: string;
  path: string;
  workType: string;
  pageCount: number;
  createdAt: string;
  artist: string | null;
  year: number | null;
  genre: string | null;
  circle: string | null;
  origin: string | null;
}

export interface Tag {
  id: number;
  name: string;
  category: string | null;
}

export interface Library {
  id: string;
  name: string;
  path: string | null;
}

export type TagSearchMode = "and" | "or";

export type SortField = "title" | "created_at";
export type SortOrder = "asc" | "desc";

export type FitMode = "screen" | "width" | "height";

export type SlideshowMode = "page" | "work";

export type ThemeMode = "light" | "dark" | "system";

export type BannerAutoClose = 0 | 1 | 3 | 5;

export type ResourceMode = "full" | "metadata_only";

export type DeleteFileAction = "delete" | "trash" | "ask";

export interface AppSettings {
  resourceMode: ResourceMode;
  directoryTemplate: string | null;
  typeLabelImage: string;
  typeLabelFolder: string;
  deleteFileAction: DeleteFileAction;
}

export interface TemplateValidation {
  valid: boolean;
  error: string | null;
}

// keep-in-sync: src-tauri/src/template.rs の WorkMetadata と対応。
// Rust側は #[serde(default)] 付きの work_type: Option<String> を追加で持つ
// （プレースホルダーレンダリング時にサーバー側で解決したtype labelを詰めるためのフィールドで、
// フロントエンドからのIPCペイロードには含まれない）。
export interface WorkMetadata {
  title: string;
  artist: string | null;
  year: number | null;
  genre: string | null;
  circle: string | null;
  origin: string | null;
}

// keep-in-sync: src-tauri/src/importer.rs の ImportMode（#[serde(rename_all = "camelCase")]）と対応。
// Rust側の Copy/Move が camelCase 化されて "copy"/"move" になる。
export type ImportMode = "copy" | "move";

export interface ImportRequest {
  sourcePath: string;
  title: string;
  artist: string | null;
  year: number | null;
  genre: string | null;
  circle: string | null;
  origin: string | null;
  mode: ImportMode;
}

export interface ImportResult {
  destinationPath: string;
  pageCount: number;
}

export interface ParsedMetadata {
  title: string;
  artist: string | null;
}

// keep-in-sync: src-tauri/src/relocator.rs の RelocationProgress enum と対応。
export type RelocationProgress =
  | { type: "started"; total: number }
  | { type: "moving"; current: number; total: number; title: string }
  | { type: "completed"; relocated: number; skipped: number; failed: number }
  | { type: "error"; message: string };

export interface RelocationPreview {
  workId: number;
  title: string;
  oldPath: string;
  newPath: string;
}

export interface DiscoveredFolder {
  path: string;
  folderName: string;
  imageCount: number;
  parsedMetadata: ParsedMetadata;
  alreadyRegistered: boolean;
}

// keep-in-sync: src-tauri/src/importer.rs の DiscoverProgress enum と対応。
export type DiscoverProgress =
  | { type: "scanning"; scannedDirs: number }
  | { type: "completed"; found: number };

// Rust側に対応するenumはなく、フロントエンド（BulkImportView）がImportQueueEventの
// 連続イベントから合成するローカル専用の進捗表現。keep-in-sync対象外。
export type BulkImportProgress =
  | { type: "started"; total: number }
  | { type: "importing"; current: number; total: number; title: string }
  | { type: "completed"; succeeded: number; failed: number }
  | { type: "error"; title: string; message: string };

export interface BulkImportSummary {
  succeeded: number;
  failed: number;
}

export interface OrphanWork {
  id: number;
  title: string;
  path: string;
  workType: string;
}

export interface UnregisteredEntry {
  path: string;
  folderName: string;
  imageCount: number;
}

export interface IntegrityReport {
  totalWorks: number;
  orphanWorks: OrphanWork[];
  unregisteredEntries: UnregisteredEntry[];
}

// keep-in-sync: src-tauri/src/integrity.rs の IntegrityCheckProgress enum と対応。
export type IntegrityCheckProgress =
  | { type: "checkingWorks"; checked: number; total: number }
  | { type: "scanningDirectory"; scannedDirs: number }
  | { type: "completed" };

// keep-in-sync: src-tauri/src/import_queue.rs の ImportQueueEvent enum と対応。
export type ImportQueueEvent =
  | { type: "enqueued"; jobId: string; total: number }
  | { type: "jobStarted"; jobId: string; total: number }
  | {
      type: "progress";
      jobId: string;
      current: number;
      total: number;
      title: string;
    }
  | { type: "itemError"; jobId: string; title: string; message: string }
  | { type: "jobCompleted"; jobId: string; succeeded: number; failed: number }
  | { type: "jobFailed"; jobId: string; message: string };

export interface EnqueueResult {
  jobId: string;
}
