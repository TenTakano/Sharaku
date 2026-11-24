import type { WorkWithHistory } from "./types";
import { formatAgo, formatSince } from "./utils/relativeTime";

/**
 * A Home shelf: which backend query fills it and how its cards are labelled.
 *
 * The SQL behind each shelf lives in the backend history queries
 * (src-tauri/src/history.rs); Home only iterates this list, so a new shelf is a
 * new command plus a new entry here.
 */
export interface ShelfDefinition {
  id: string;
  title: string;
  /** One-line description of the rule under the title. */
  rule: string;
  /** Tauri command taking `{ limit }` and returning `WorkWithHistory[]` for the active library. */
  command: string;
  /** Second line under the card title. */
  meta: (work: WorkWithHistory, now: Date) => string;
  /** Badge on the cover's top-left corner; no badge when it returns "". */
  badge?: (work: WorkWithHistory, index: number, now: Date) => string;
  /** Shows "引き直す", which runs the query again. */
  canShuffle?: boolean;
}

/** Number of works loaded per shelf. */
export const SHELF_LIMIT = 20;

function viewCount(work: WorkWithHistory): string {
  return `${work.viewCount} 回`;
}

function addedAgo(work: WorkWithHistory, now: Date): string {
  return `${formatAgo(work.createdAt, now)}に追加`;
}

export const SHELVES: ShelfDefinition[] = [
  {
    id: "recently-viewed",
    title: "最近見た",
    rule: "閲覧した順",
    command: "list_recently_viewed_works",
    meta: (w, now) => (w.lastViewedAt ? formatAgo(w.lastViewedAt, now) : ""),
  },
  {
    id: "pickup",
    title: "ピックアップ",
    rule: "よく見ていたのに、しばらく開いていない作品",
    command: "list_pickup_works",
    meta: (w) => `前は ${viewCount(w)}`,
    badge: (w, _i, now) =>
      w.lastViewedAt ? formatSince(w.lastViewedAt, now) : "",
  },
  {
    id: "recently-added",
    title: "最近追加した",
    rule: "取り込んだ順",
    command: "list_recently_added_works",
    meta: addedAgo,
  },
  {
    id: "frequently-viewed",
    title: "よく見ている",
    rule: "閲覧回数の多い順",
    command: "list_frequently_viewed_works",
    meta: viewCount,
    badge: (_w, i) => (i < 3 ? `No.${i + 1}` : ""),
  },
  {
    id: "unviewed",
    title: "まだ見ていない",
    rule: "一度も開いていない作品",
    command: "list_unviewed_works",
    meta: addedAgo,
    badge: () => "未読",
  },
  {
    id: "random",
    title: "ランダム",
    rule: "全作品からおまかせ",
    command: "list_random_works",
    meta: (w, now) =>
      w.lastViewedAt ? `最終閲覧 ${formatAgo(w.lastViewedAt, now)}` : "未読",
    canShuffle: true,
  },
];
