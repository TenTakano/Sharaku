import { invoke } from "@tauri-apps/api/core";

const cache = new Map<number, string>();

export function getCachedThumbnail(workId: number): string | undefined {
  return cache.get(workId);
}

export function setCachedThumbnail(workId: number, url: string): void {
  cache.set(workId, url);
}

export function clearThumbnailCache(): void {
  for (const url of cache.values()) {
    URL.revokeObjectURL(url);
  }
  cache.clear();
}

/** Returns the cached thumbnail URL of a work, fetching it on a miss; null when it has none. */
export async function loadThumbnail(workId: number): Promise<string | null> {
  const cached = cache.get(workId);
  if (cached) return cached;
  try {
    const bytes: number[] = await invoke("get_thumbnail", { workId });
    const blob = new Blob([new Uint8Array(bytes)], { type: "image/webp" });
    const url = URL.createObjectURL(blob);
    cache.set(workId, url);
    return url;
  } catch {
    return null;
  }
}
