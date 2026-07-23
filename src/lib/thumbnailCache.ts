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
