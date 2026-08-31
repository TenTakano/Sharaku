export type TimerHandle = ReturnType<typeof setTimeout> | null;

export type ViewKind =
  | "library"
  | "viewer"
  | "settings"
  | "import"
  | "bulk-import"
  | "add-library"
  | "app-settings"
  | "playlist";

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

export interface Playlist {
  id: number;
  name: string;
}

export interface PlaylistItem {
  workId: number;
  title: string;
  workType: string;
  pageCount: number;
  createdAt: string;
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

// keep-in-sync: corresponds to WorkMetadata in src-tauri/src/template.rs.
// The Rust side additionally has work_type: Option<String> with #[serde(default)]
// (a field populated server-side with the resolved type label during placeholder
// rendering; it is not included in the frontend's IPC payload).
export interface WorkMetadata {
  title: string;
  artist: string | null;
  year: number | null;
  genre: string | null;
  circle: string | null;
  origin: string | null;
}

export type ImportKind = "folder" | "image";

export type DropKind = "folder" | "image" | "other";

export interface ImportRequest {
  sourcePath: string;
  title: string;
  artist: string | null;
  year: number | null;
  genre: string | null;
  circle: string | null;
  origin: string | null;
  kind: ImportKind;
}

export interface ImportResult {
  destinationPath: string;
  pageCount: number;
}

export interface ParsedMetadata {
  title: string;
  artist: string | null;
}

export interface DiscoveredFolder {
  path: string;
  folderName: string;
  imageCount: number;
  parsedMetadata: ParsedMetadata;
  alreadyRegistered: boolean;
}

export interface DiscoveredImage {
  path: string;
  fileName: string;
  parsedMetadata: ParsedMetadata;
  alreadyRegistered: boolean;
}

export interface SkippedFolder {
  path: string;
  folderName: string;
  imageCount: number;
}

export interface DiscoverResult {
  folders: DiscoveredFolder[];
  images: DiscoveredImage[];
  skippedFolders: SkippedFolder[];
}

// keep-in-sync: corresponds to the DiscoverProgress enum in src-tauri/src/importer.rs.
export type DiscoverProgress =
  | { type: "scanning"; scannedDirs: number }
  | { type: "completed"; found: number };

// No corresponding enum on the Rust side; this is a local-only progress representation
// synthesized by the frontend (BulkImportView) from a sequence of ImportQueueEvent events.
// Not subject to keep-in-sync.
export type BulkImportProgress =
  | { type: "started"; total: number }
  | { type: "importing"; current: number; total: number; title: string }
  | { type: "completed"; succeeded: number; failed: number }
  | { type: "error"; title: string; message: string };

export interface BulkImportSummary {
  succeeded: number;
  failed: number;
}

// keep-in-sync: corresponds to the ImportQueueEvent enum in src-tauri/src/import_queue.rs.
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
