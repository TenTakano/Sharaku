<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { VList } from "virtua/svelte";
  import WorkCardComponent from "./WorkCard.svelte";
  import { WorkCard } from "./WorkCard.svelte";
  import type { WorkSummary, SortField, SortOrder } from "../types";

  interface Props {
    reloadTrigger: number;
    onSelectWork: (workId: number) => void;
  }

  let { reloadTrigger, onSelectWork }: Props = $props();

  let works = $state<WorkSummary[]>([]);
  let sortField = $state<SortField>("created_at");
  let sortOrder = $state<SortOrder>("desc");
  let containerWidth = $state(0);

  const CARD_WIDTH = 180;
  const GAP = 16;

  let columnCount = $derived(
    Math.max(1, Math.floor((containerWidth + GAP) / (CARD_WIDTH + GAP))),
  );

  let rows = $derived.by(() => {
    const result: WorkSummary[][] = [];
    for (let i = 0; i < works.length; i += columnCount) {
      result.push(works.slice(i, i + columnCount));
    }
    return result;
  });

  async function loadWorks() {
    WorkCard.clearCache();
    works = await invoke("list_works", {
      sortBy: sortField,
      sortOrder: sortOrder,
    });
  }

  $effect(() => {
    void reloadTrigger;
    void sortField;
    void sortOrder;
    loadWorks();
  });

  function handleSort(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    switch (value) {
      case "created_at_desc":
        sortField = "created_at";
        sortOrder = "desc";
        break;
      case "created_at_asc":
        sortField = "created_at";
        sortOrder = "asc";
        break;
      case "title_asc":
        sortField = "title";
        sortOrder = "asc";
        break;
      case "title_desc":
        sortField = "title";
        sortOrder = "desc";
        break;
    }
  }
</script>

<div class="toolbar">
  <div class="sort-control">
    <label for="sort-select">Sort:</label>
    <select
      id="sort-select"
      value="{sortField}_{sortOrder}"
      onchange={handleSort}
    >
      <option value="created_at_desc">Date (Newest)</option>
      <option value="created_at_asc">Date (Oldest)</option>
      <option value="title_asc">Title (A-Z)</option>
      <option value="title_desc">Title (Z-A)</option>
    </select>
  </div>
  <span class="work-count">{works.length} works</span>
</div>

<div class="grid-container" bind:clientWidth={containerWidth}>
  {#if rows.length > 0}
    <VList data={rows} getKey={(_, i) => i} itemSize={280}>
      {#snippet children(row)}
        <div
          class="grid-row"
          style="gap: {GAP}px; grid-template-columns: repeat({columnCount}, {CARD_WIDTH}px);"
        >
          {#each row as work (work.id)}
            <WorkCardComponent {work} onclick={onSelectWork} />
          {/each}
        </div>
      {/snippet}
    </VList>
  {:else}
    <div class="empty-state">
      <p>No works found. Scan a directory to get started.</p>
    </div>
  {/if}
</div>
