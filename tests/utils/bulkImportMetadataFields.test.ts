import { describe, it, expect } from "vitest";
import { BULK_IMPORT_METADATA_FIELDS } from "../../src/lib/utils/bulkImportMetadataFields";

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
