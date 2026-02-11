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
