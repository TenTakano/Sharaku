import { describe, it, expect } from "vitest";
import {
  BULK_IMPORT_METADATA_FIELDS,
  extractMetadataPlaceholders,
} from "../../src/lib/utils/templatePlaceholders";

describe("BULK_IMPORT_METADATA_FIELDS", () => {
  it("year/genre/circle/origin の4種のみを対象とする（title/artist/typeは含まない）", () => {
    expect(BULK_IMPORT_METADATA_FIELDS).toEqual([
      "year",
      "genre",
      "circle",
      "origin",
    ]);
  });
});

describe("extractMetadataPlaceholders", () => {
  it("テンプレートがnullの場合は空配列を返す", () => {
    expect(extractMetadataPlaceholders(null)).toEqual([]);
  });

  it("プレースホルダーを含まないテンプレートは空配列を返す", () => {
    expect(extractMetadataPlaceholders("{title}")).toEqual([]);
  });

  it("対象プレースホルダーを含むテンプレートから抽出する", () => {
    expect(extractMetadataPlaceholders("{circle}/{artist}/{title}")).toEqual([
      "circle",
    ]);
  });

  it("複数の対象プレースホルダーを固定順で返す", () => {
    expect(
      extractMetadataPlaceholders("{origin}/{year}/{circle}/{title}"),
    ).toEqual(["year", "circle", "origin"]);
  });

  it("重複するプレースホルダーはdedupする", () => {
    expect(extractMetadataPlaceholders("{circle}/{title}/{circle}")).toEqual([
      "circle",
    ]);
  });

  it("title/artist/typeは抽出対象から除外する", () => {
    expect(extractMetadataPlaceholders("{type}/{artist}/{title}")).toEqual([]);
  });

  it("全ての対象プレースホルダーを含むテンプレートから全件抽出する", () => {
    expect(
      extractMetadataPlaceholders(
        "{year}/{genre}/{circle}/{origin}/{artist}/{title}",
      ),
    ).toEqual(["year", "genre", "circle", "origin"]);
  });
});
