import { describe, it, expect } from "vitest";
import { formatAgo, formatSince } from "../../src/lib/utils/relativeTime";

const NOW = new Date("2026-06-15T12:00:00.000Z");

function before(ms: number): string {
  return new Date(NOW.getTime() - ms).toISOString();
}

const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

describe("formatAgo", () => {
  it.each([
    [10 * 1000, "たった今"],
    [5 * MINUTE, "5分前"],
    [2 * HOUR, "2時間前"],
    [1 * DAY + HOUR, "昨日"],
    [3 * DAY, "3日前"],
    [8 * DAY, "1週間前"],
    [15 * DAY, "2週間前"],
    [125 * DAY, "4か月前"],
    [400 * DAY, "1年前"],
  ])("%i ms 前は「%s」", (ms, expected) => {
    expect(formatAgo(before(ms), NOW)).toBe(expected);
  });

  it("未来の日時はたった今として扱う", () => {
    expect(formatAgo(new Date(NOW.getTime() + HOUR).toISOString(), NOW)).toBe(
      "たった今",
    );
  });

  it("解釈できない文字列は空文字を返す", () => {
    expect(formatAgo("invalid", NOW)).toBe("");
  });
});

describe("formatSince", () => {
  it.each([
    [20 * DAY, "2週間ぶり"],
    [125 * DAY, "4か月ぶり"],
    [800 * DAY, "2年ぶり"],
  ])("%i ms 前は「%s」", (ms, expected) => {
    expect(formatSince(before(ms), NOW)).toBe(expected);
  });
});

describe("formatSince（1日未満）", () => {
  it("1日未満は空文字を返す", () => {
    expect(formatSince(before(3 * HOUR), NOW)).toBe("");
  });
});
