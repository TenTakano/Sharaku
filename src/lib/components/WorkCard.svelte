<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCachedThumbnail, setCachedThumbnail } from "../thumbnailCache";
  import type { WorkSummary } from "../types";

  interface Props {
    work: WorkSummary;
    onclick: (workId: number) => void;
    oncontextmenu?: (workId: number, e: MouseEvent) => void;
  }

  let { work, onclick, oncontextmenu }: Props = $props();
  let thumbnailUrl = $state<string | null>(null);
  let loading = $state(true);

  async function loadThumbnail() {
    const cached = getCachedThumbnail(work.id);
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
      setCachedThumbnail(work.id, url);
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

<button
  class="work-card"
  onclick={() => onclick(work.id)}
  oncontextmenu={(e) => {
    if (oncontextmenu) {
      e.preventDefault();
      oncontextmenu(work.id, e);
    }
  }}
>
  {#if loading}
    <div class="no-thumbnail"></div>
  {:else if thumbnailUrl}
    <img src={thumbnailUrl} alt={work.title} />
  {:else}
    <div class="no-thumbnail">No Image</div>
  {/if}
  <span class="work-title">{work.title}</span>
</button>
