<script lang="ts">
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import WorkCard from "./WorkCard.svelte";
  import type { PlaylistItem, WorkSummary } from "../types";

  interface Props {
    item: PlaylistItem;
    index: number;
    onSelectWork: (workId: number) => void;
    onRemove: (item: PlaylistItem) => void;
  }

  let { item, index, onSelectWork, onRemove }: Props = $props();

  const sortable = createSortable({
    id: item.workId,
    get index() {
      return index;
    },
    group: "playlist-items",
  });

  let work = $derived<WorkSummary>({
    id: item.workId,
    title: item.title,
    workType: item.workType,
    pageCount: item.pageCount,
    createdAt: item.createdAt,
  });
</script>

<div
  class="playlist-item"
  class:dragging={sortable.isDragging}
  {@attach sortable.attach}
>
  <span
    class="playlist-item-handle"
    {@attach sortable.attachHandle}
    role="button"
    tabindex="0"
    aria-label="ドラッグして並び替え"
  >
    &#10021;
  </span>
  <div class="playlist-item-card">
    <WorkCard {work} onclick={onSelectWork} />
  </div>
  <button
    class="playlist-item-remove"
    onclick={() => onRemove(item)}
    title="プレイリストから削除"
    aria-label="プレイリストから削除"
  >
    &times;
  </button>
</div>
