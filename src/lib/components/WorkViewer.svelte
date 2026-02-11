<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { WorkDetail } from "../types";

  interface Props {
    workId: number;
    onBack: () => void;
  }

  let { workId, onBack }: Props = $props();

  let work = $state<WorkDetail | null>(null);
  let imageUrl = $state<string | null>(null);
  let error = $state<string | null>(null);

  async function loadWork() {
    try {
      work = await invoke("get_work", { workId });
      const bytes: number[] = await invoke("read_image_file", {
        path: work!.path,
      });
      const blob = new Blob([new Uint8Array(bytes)]);
      imageUrl = URL.createObjectURL(blob);
    } catch (e) {
      error = String(e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onBack();
    }
  }

  $effect(() => {
    void workId;
    loadWork();
    return () => {
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
      }
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="viewer-overlay">
  <div class="viewer-header">
    <button class="viewer-back-btn" onclick={onBack}>← Back</button>
    {#if work}
      <span class="viewer-title">{work.title}</span>
    {/if}
  </div>
  <div class="viewer-content">
    {#if error}
      <p class="viewer-error">{error}</p>
    {:else if imageUrl}
      <img class="viewer-image" src={imageUrl} alt={work?.title ?? ""} />
    {:else}
      <p class="viewer-loading">Loading...</p>
    {/if}
  </div>
</div>
