<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { WorkSummary } from "../types";

  interface Props {
    work: WorkSummary;
    onclick: (workId: number) => void;
  }

  let { work, onclick }: Props = $props();
  let thumbnailUrl = $state<string | null>(null);
  let loading = $state(true);

  const cache = WorkCard._thumbnailCache;

  async function loadThumbnail() {
    const cached = cache.get(work.id);
    if (cached) {
      thumbnailUrl = cached;
      loading = false;
      return;
    }
    try {
      const bytes: number[] = await invoke("get_thumbnail", {
        workId: work.id,
      });
      const blob = new Blob([new Uint8Array(bytes)], { type: "image/webp" });
      const url = URL.createObjectURL(blob);
      cache.set(work.id, url);
      thumbnailUrl = url;
    } catch {
      thumbnailUrl = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void work.id;
    loading = true;
    loadThumbnail();
  });
</script>

<script lang="ts" module>
  export class WorkCard {
    static _thumbnailCache = new Map<number, string>();

    static clearCache() {
      for (const url of WorkCard._thumbnailCache.values()) {
        URL.revokeObjectURL(url);
      }
      WorkCard._thumbnailCache.clear();
    }
  }
</script>

<button class="work-card" onclick={() => onclick(work.id)}>
  {#if loading}
    <div class="no-thumbnail"></div>
  {:else if thumbnailUrl}
    <img src={thumbnailUrl} alt={work.title} />
  {:else}
    <div class="no-thumbnail">No Image</div>
  {/if}
  <span class="work-title">{work.title}</span>
</button>
