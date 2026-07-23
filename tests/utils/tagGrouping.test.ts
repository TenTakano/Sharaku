import { describe, it, expect } from "vitest";
import { groupTagsByCategory } from "../../src/lib/utils/tagGrouping";
import type { Tag } from "../../src/lib/types";

function tag(id: number, name: string, category: string | null): Tag {
  return { id, name, category };
}

describe("groupTagsByCategory", () => {
  it("空配列を渡すと空配列を返す", () => {
    expect(groupTagsByCategory([])).toEqual([]);
  });

  it("同じカテゴリのタグを1グループにまとめる", () => {
    const tags = [tag(1, "作者A", "artist"), tag(2, "作者B", "artist")];

    const result = groupTagsByCategory(tags);

    expect(result).toHaveLength(1);
    expect(result[0].category).toBe("artist");
    expect(result[0].displayName).toBe("artist");
    expect(result[0].tags).toEqual(tags);
  });

  it("カテゴリごとに別グループへ分ける", () => {
    const artistTag = tag(1, "作者A", "artist");
    const genreTag = tag(2, "ジャンルA", "genre");

    const result = groupTagsByCategory([artistTag, genreTag]);

    expect(result).toHaveLength(2);
    expect(result.map((g) => g.category)).toEqual(["artist", "genre"]);
    expect(result[0].tags).toEqual([artistTag]);
    expect(result[1].tags).toEqual([genreTag]);
  });

  it("category が null のタグは other 表示名のグループにまとめる", () => {
    const result = groupTagsByCategory([tag(1, "無分類", null)]);

    expect(result).toHaveLength(1);
    expect(result[0].category).toBeNull();
    expect(result[0].displayName).toBe("other");
  });

  it("タグの出現順にグループを保持する", () => {
    const tags = [
      tag(1, "ジャンルA", "genre"),
      tag(2, "作者A", "artist"),
      tag(3, "ジャンルB", "genre"),
    ];

    const result = groupTagsByCategory(tags);

    expect(result.map((g) => g.category)).toEqual(["genre", "artist"]);
    expect(result[0].tags.map((t) => t.id)).toEqual([1, 3]);
  });
});
