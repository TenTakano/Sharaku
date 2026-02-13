export type ScanProgress =
  | { type: "started"; total: number }
  | { type: "processing"; current: number; total: number; fileName: string }
  | { type: "completed"; registered: number; failed: number }
  | { type: "error"; message: string };

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
}
