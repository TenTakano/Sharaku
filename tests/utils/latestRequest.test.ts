import { describe, it, expect } from "vitest";
import { createLatestRequestGuard } from "../../src/lib/utils/latestRequest";

describe("createLatestRequestGuard", () => {
  it("next() は呼ぶたびに増加するIDを返す", () => {
    const guard = createLatestRequestGuard();

    expect(guard.next()).toBe(1);
    expect(guard.next()).toBe(2);
    expect(guard.next()).toBe(3);
  });

  it("最後に発行したIDのみ isLatest が true を返す", () => {
    const guard = createLatestRequestGuard();

    const first = guard.next();
    const second = guard.next();

    expect(guard.isLatest(first)).toBe(false);
    expect(guard.isLatest(second)).toBe(true);
  });

  it("まだ next() を呼んでいない場合は isLatest(0) が true を返す", () => {
    const guard = createLatestRequestGuard();

    expect(guard.isLatest(0)).toBe(true);
  });

  it("古いリクエストのIDが新しいリクエスト発行後に isLatest false になる", () => {
    const guard = createLatestRequestGuard();

    const stale = guard.next();
    expect(guard.isLatest(stale)).toBe(true);

    guard.next();
    expect(guard.isLatest(stale)).toBe(false);
  });
});
