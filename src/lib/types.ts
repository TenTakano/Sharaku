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
}

export type SortField = "title" | "created_at";
export type SortOrder = "asc" | "desc";
