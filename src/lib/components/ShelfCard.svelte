<script lang="ts">
  import { loadThumbnail } from "../thumbnailCache";
  import type { WorkWithHistory } from "../types";

  interface Props {
    work: WorkWithHistory;
    meta: string;
    badge?: string;
    onclick: (workId: number) => void;
  }

  let { work, meta, badge = "", onclick }: Props = $props();
  let thumbnailUrl = $state<string | null>(null);

  $effect(() => {
    const id = work.id;
    thumbnailUrl = null;
    loadThumbnail(id).then((url) => {
      if (work.id === id) thumbnailUrl = url;
    });
  });
</script>

<button class="shelf-card" onclick={() => onclick(work.id)}>
  <span class="shelf-card-cover">
    {#if thumbnailUrl}
      <img src={thumbnailUrl} alt="" />
    {/if}
    {#if badge}
      <span class="shelf-card-badge">{badge}</span>
    {/if}
    <span class="shelf-card-pages">{work.pageCount} 枚</span>
  </span>
  <span class="shelf-card-title">{work.title}</span>
  {#if meta}
    <span class="shelf-card-meta">{meta}</span>
  {/if}
</button>
