const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

type Span = { value: number; unit: string };

/** Rounds the time between `iso` and `now` down to the largest whole unit. */
function spanSince(iso: string, now: Date): Span | null {
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return null;
  const elapsed = Math.max(0, now.getTime() - then);
  const days = Math.floor(elapsed / DAY);
  if (days >= 365) return { value: Math.floor(days / 365), unit: "年" };
  if (days >= 30) return { value: Math.floor(days / 30), unit: "か月" };
  if (days >= 7) return { value: Math.floor(days / 7), unit: "週間" };
  if (days >= 1) return { value: days, unit: "日" };
  const hours = Math.floor(elapsed / HOUR);
  if (hours >= 1) return { value: hours, unit: "時間" };
  return { value: Math.floor(elapsed / MINUTE), unit: "分" };
}

/** "3日前", "2週間前", "1年前"; "たった今" under a minute and "昨日" for one day. */
export function formatAgo(iso: string, now: Date = new Date()): string {
  const span = spanSince(iso, now);
  if (!span) return "";
  if (span.unit === "分" && span.value === 0) return "たった今";
  if (span.unit === "日" && span.value === 1) return "昨日";
  return `${span.value}${span.unit}前`;
}

/** "4か月ぶり", "1年ぶり": how long a work has gone unopened ("" under a day). */
export function formatSince(iso: string, now: Date = new Date()): string {
  const span = spanSince(iso, now);
  if (!span || span.unit === "分" || span.unit === "時間") return "";
  return `${span.value}${span.unit}ぶり`;
}
