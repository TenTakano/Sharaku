export interface Work {
  id: number;
  title: string;
  thumbnail: number[];
}

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

export type SortField = "title" | "created_at";
export type SortOrder = "asc" | "desc";

export type FitMode = "screen" | "width" | "height";

export type SlideshowMode = "page" | "work";

export interface AppSettings {
  libraryRoot: string | null;
  directoryTemplate: string | null;
  typeLabelImage: string;
  typeLabelFolder: string;
}

export interface TemplateValidation {
  valid: boolean;
  error: string | null;
}

export interface WorkMetadata {
  title: string;
  artist: string | null;
  year: number | null;
  genre: string | null;
  circle: string | null;
  origin: string | null;
}

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

export type DiscoverProgress =
  | { type: "scanning"; scannedDirs: number }
  | { type: "completed"; found: number };

export type BulkImportProgress =
  | { type: "started"; total: number }
  | { type: "importing"; current: number; total: number; title: string }
  | { type: "completed"; succeeded: number; failed: number };

export interface BulkImportSummary {
  succeeded: number;
  failed: number;
}
