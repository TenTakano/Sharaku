<script lang="ts">
  import ShelfCard from "./ShelfCard.svelte";
  import type { ShelfDefinition } from "../shelves";
  import type { WorkWithHistory } from "../types";

  interface Props {
    shelf: ShelfDefinition;
    works: WorkWithHistory[];
    now: Date;
    onOpenWork: (workIds: number[], workId: number) => void;
    onBrowseAll: () => void;
    onShuffle?: () => void;
  }

  let { shelf, works, now, onOpenWork, onBrowseAll, onShuffle }: Props =
    $props();

  let workIds = $derived(works.map((w) => w.id));
  let headingId = $derived(`shelf-${shelf.id}`);
</script>

<section class="shelf" aria-labelledby={headingId}>
  <div class="shelf-header">
    <div class="shelf-heading">
      <h2 id={headingId} class="shelf-title">{shelf.title}</h2>
      <span class="shelf-rule">{shelf.rule}</span>
    </div>
    <div class="shelf-actions">
      {#if shelf.canShuffle && onShuffle}
        <button class="shelf-action shelf-action-outline" onclick={onShuffle}>
          引き直す
        </button>
      {/if}
      <button
        class="shelf-action"
        onclick={() => onOpenWork(workIds, workIds[0])}
      >
        すべて再生
      </button>
      <button class="shelf-action shelf-action-muted" onclick={onBrowseAll}>
        すべて見る
      </button>
    </div>
  </div>
  <div class="shelf-items">
    {#each works as work, i (work.id)}
      <ShelfCard
        {work}
        meta={shelf.meta(work, now)}
        badge={shelf.badge?.(work, i, now) ?? ""}
        onclick={(id) => onOpenWork(workIds, id)}
      />
    {/each}
  </div>
</section>
