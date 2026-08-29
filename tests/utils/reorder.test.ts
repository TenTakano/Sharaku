import { describe, it, expect } from "vitest";
import { moveItem } from "../../src/lib/utils/reorder";

describe("moveItem", () => {
  it("先頭の要素を末尾へ移動する", () => {
    expect(moveItem([1, 2, 3, 4], 1, 4)).toEqual([2, 3, 4, 1]);
  });

  it("末尾の要素を先頭へ移動する", () => {
    expect(moveItem([1, 2, 3, 4], 4, 1)).toEqual([4, 1, 2, 3]);
  });

  it("中間の要素を後方へ移動する", () => {
    expect(moveItem([1, 2, 3, 4, 5], 2, 4)).toEqual([1, 3, 4, 2, 5]);
  });

  it("中間の要素を前方へ移動する", () => {
    expect(moveItem([1, 2, 3, 4, 5], 4, 2)).toEqual([1, 4, 2, 3, 5]);
  });

  it("移動元と移動先が同じ場合は元の配列と同じ順序を返す", () => {
    expect(moveItem([1, 2, 3], 2, 2)).toEqual([1, 2, 3]);
  });

  it("元の配列を変更しない", () => {
    const original = [1, 2, 3];
    moveItem(original, 1, 3);
    expect(original).toEqual([1, 2, 3]);
  });

  it("存在しない fromItem を渡すと元の配列を返す", () => {
    expect(moveItem([1, 2, 3], 99, 1)).toEqual([1, 2, 3]);
  });

  it("存在しない toItem を渡すと元の配列を返す", () => {
    expect(moveItem([1, 2, 3], 1, 99)).toEqual([1, 2, 3]);
  });
});
