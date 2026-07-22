import { describe, it, expect, vi, afterEach } from "vitest";
import {
  getCachedThumbnail,
  setCachedThumbnail,
  clearThumbnailCache,
} from "../../src/lib/thumbnailCache";

describe("thumbnailCache", () => {
  afterEach(() => {
    clearThumbnailCache();
  });

  it("未登録の workId に対して undefined を返す", () => {
    expect(getCachedThumbnail(999)).toBeUndefined();
  });

  it("setCachedThumbnail で登録した値を getCachedThumbnail で取得できる", () => {
    setCachedThumbnail(1, "blob:thumbnail-1");

    expect(getCachedThumbnail(1)).toBe("blob:thumbnail-1");
  });

  it("同じ workId に再登録すると値が上書きされる", () => {
    setCachedThumbnail(1, "blob:thumbnail-1");
    setCachedThumbnail(1, "blob:thumbnail-2");

    expect(getCachedThumbnail(1)).toBe("blob:thumbnail-2");
  });

  it("clearThumbnailCache はキャッシュ済みの全URLを revokeObjectURL する", () => {
    const revokeObjectURL = vi
      .spyOn(URL, "revokeObjectURL")
      .mockImplementation(() => {});

    setCachedThumbnail(1, "blob:thumbnail-1");
    setCachedThumbnail(2, "blob:thumbnail-2");

    clearThumbnailCache();

    expect(revokeObjectURL).toHaveBeenCalledWith("blob:thumbnail-1");
    expect(revokeObjectURL).toHaveBeenCalledWith("blob:thumbnail-2");
    expect(revokeObjectURL).toHaveBeenCalledTimes(2);

    revokeObjectURL.mockRestore();
  });

  it("clearThumbnailCache 実行後はキャッシュが空になる", () => {
    vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});
    setCachedThumbnail(1, "blob:thumbnail-1");

    clearThumbnailCache();

    expect(getCachedThumbnail(1)).toBeUndefined();

    vi.restoreAllMocks();
  });
});
