<script lang="ts">
  import { loadThumbnail as fetchThumbnail } from "../thumbnailCache";
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
    thumbnailUrl = await fetchThumbnail(work.id);
    loading = false;
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
