import type { Tag } from "../types";

export interface TagsByCategory {
  category: string | null;
  displayName: string;
  tags: Tag[];
}

export function groupTagsByCategory(tags: Tag[]): TagsByCategory[] {
  const map = new Map<string | null, Tag[]>();
  for (const tag of tags) {
    const key = tag.category ?? null;
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(tag);
  }
  const result: TagsByCategory[] = [];
  for (const [category, categoryTags] of map) {
    result.push({
      category,
      displayName: category ?? "other",
      tags: categoryTags,
    });
  }
  return result;
}
