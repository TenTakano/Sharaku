export type BulkImportMetadataField = "year" | "genre" | "circle" | "origin";

// keep-in-sync: this set must stay a subset of KNOWN_PLACEHOLDERS in
// src-tauri/src/template.rs, excluding title/artist/type which are not
// bulk-import metadata columns. Covered by templatePlaceholders.test.ts.
export const BULK_IMPORT_METADATA_FIELDS: BulkImportMetadataField[] = [
  "year",
  "genre",
  "circle",
  "origin",
];

const PLACEHOLDER_RE = /\{([a-zA-Z_]+)\}/g;

export function extractMetadataPlaceholders(
  template: string | null,
): BulkImportMetadataField[] {
  if (!template) return [];

  const found = new Set<string>();
  for (const match of template.matchAll(PLACEHOLDER_RE)) {
    found.add(match[1]);
  }

  return BULK_IMPORT_METADATA_FIELDS.filter((field) => found.has(field));
}
