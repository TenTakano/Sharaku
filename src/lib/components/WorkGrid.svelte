<script lang="ts">
  import { getDatabase } from "../db/database";
  import type { Work } from "../types";

  interface Props {
    reloadTrigger: number;
  }

  let { reloadTrigger }: Props = $props();
  let works = $state<Work[]>([]);

  function thumbnailToDataUrl(bytes: number[]): string {
    const u8 = new Uint8Array(bytes);
    const chunk = 8192;
    let binary = "";
    for (let i = 0; i < u8.length; i += chunk) {
      binary += String.fromCharCode(...u8.subarray(i, i + chunk));
    }
    return `data:image/webp;base64,${btoa(binary)}`;
  }

  async function loadWorks() {
    const db = await getDatabase();
    works = await db.select<Work[]>(
      "SELECT id, title, thumbnail FROM works ORDER BY created_at DESC"
    );
  }

  $effect(() => {
    void reloadTrigger;
    loadWorks();
  });
</script>

<div class="work-grid">
  {#each works as work (work.id)}
    <div class="work-card">
      {#if work.thumbnail && work.thumbnail.length > 0}
        <img src={thumbnailToDataUrl(work.thumbnail)} alt={work.title} />
      {:else}
        <div class="no-thumbnail">No Image</div>
      {/if}
      <span class="work-title">{work.title}</span>
    </div>
  {/each}
</div>
