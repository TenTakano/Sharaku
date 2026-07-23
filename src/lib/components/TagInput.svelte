<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { debounce } from "../utils/debounce";
  import { createLatestRequestGuard } from "../utils/latestRequest";
  import type { Tag } from "../types";

  interface Props {
    placeholder?: string;
    variant?: "light" | "dark";
    onSelectTag: (tag: Tag) => void;
    onCreateTag?: (name: string) => Promise<Tag>;
    excludeTagIds?: number[];
  }

  let {
    placeholder = "タグを追加...",
    variant = "light",
    onSelectTag,
    onCreateTag,
    excludeTagIds = [],
  }: Props = $props();

  let query = $state("");
  let suggestions = $state<Tag[]>([]);
  let open = $state(false);
  let highlightIndex = $state(-1);
  let inputEl = $state<HTMLInputElement | null>(null);

  const searchRequestGuard = createLatestRequestGuard();

  async function search(q: string) {
    if (!q.trim()) {
      suggestions = [];
      open = false;
      return;
    }
    const requestId = searchRequestGuard.next();
    try {
      const results: Tag[] = await invoke("search_tags", {
        query: q.trim(),
        category: null,
      });
      if (!searchRequestGuard.isLatest(requestId)) return;
      suggestions = results.filter((t) => !excludeTagIds.includes(t.id));
      open = true;
      highlightIndex = -1;
    } catch {
      if (!searchRequestGuard.isLatest(requestId)) return;
      suggestions = [];
    }
  }

  const debouncedSearch = debounce((q: string) => search(q), 200);

  function handleInput(e: Event) {
    query = (e.target as HTMLInputElement).value;
    debouncedSearch(query);
  }

  function selectTag(tag: Tag) {
    onSelectTag(tag);
    query = "";
    suggestions = [];
    open = false;
    highlightIndex = -1;
  }

  async function createTag(name: string) {
    if (!onCreateTag) return;
    const tag = await onCreateTag(name);
    selectTag(tag);
  }

  const CATEGORY_ORDER: Record<string, number> = {
    artist: 0,
    circle: 1,
    genre: 2,
    series: 3,
  };

  type GroupedItem =
    | { type: "header"; category: string }
    | { type: "tag"; tag: Tag; flatIndex: number };

  let groupedItems = $derived.by(() => {
    if (suggestions.length === 0) return [];

    const sorted = [...suggestions].sort((a, b) => {
      const catA = a.category ?? "\uffff";
      const catB = b.category ?? "\uffff";
      const orderA = CATEGORY_ORDER[catA] ?? 999;
      const orderB = CATEGORY_ORDER[catB] ?? 999;
      if (orderA !== orderB) return orderA - orderB;
      return a.name.localeCompare(b.name);
    });

    const items: GroupedItem[] = [];
    let currentCategory: string | undefined;
    let flatIndex = 0;
    for (const tag of sorted) {
      const cat = tag.category ?? "";
      if (cat !== currentCategory) {
        currentCategory = cat;
        items.push({ type: "header", category: tag.category ?? "" });
      }
      items.push({ type: "tag", tag, flatIndex });
      flatIndex++;
    }
    return items;
  });

  let sortedSuggestions = $derived(
    groupedItems
      .filter(
        (item): item is Extract<GroupedItem, { type: "tag" }> =>
          item.type === "tag",
      )
      .map((item) => item.tag),
  );

  let hasExactMatch = $derived(
    suggestions.some(
      (t) => t.name.toLowerCase() === query.trim().toLowerCase(),
    ),
  );
  let showCreate = $derived(
    open && query.trim() && !hasExactMatch && !!onCreateTag,
  );
  let totalOptions = $derived(sortedSuggestions.length + (showCreate ? 1 : 0));

  function handleKeydown(e: KeyboardEvent) {
    if (!open && e.key !== "Escape") return;
    if (totalOptions === 0 && e.key !== "Escape") return;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        highlightIndex = (highlightIndex + 1) % totalOptions;
        break;
      case "ArrowUp":
        e.preventDefault();
        highlightIndex =
          highlightIndex <= 0 ? totalOptions - 1 : highlightIndex - 1;
        break;
      case "Enter":
        e.preventDefault();
        if (highlightIndex >= 0 && highlightIndex < sortedSuggestions.length) {
          selectTag(sortedSuggestions[highlightIndex]);
        } else if (showCreate && highlightIndex === sortedSuggestions.length) {
          createTag(query.trim());
        }
        break;
      case "Escape":
        open = false;
        highlightIndex = -1;
        inputEl?.blur();
        break;
    }
  }

  function handleFocusOut() {
    setTimeout(() => {
      open = false;
      highlightIndex = -1;
    }, 150);
  }
</script>

<div class="tag-input-wrapper tag-input-{variant}">
  <input
    bind:this={inputEl}
    type="text"
    class="tag-input-field"
    {placeholder}
    value={query}
    oninput={handleInput}
    onkeydown={handleKeydown}
    onfocusin={() => {
      if (query.trim()) search(query);
    }}
    onfocusout={handleFocusOut}
  />
  {#if open && totalOptions > 0}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="tag-input-dropdown" onmousedown={(e) => e.preventDefault()}>
      {#each groupedItems as item (item.type === "header" ? `h:${item.category}` : `t:${item.tag.id}`)}
        {#if item.type === "header"}
          <div class="tag-input-category-header">
            {item.category || "その他"}
          </div>
        {:else}
          <button
            class="tag-input-option"
            class:tag-input-option-highlight={item.flatIndex === highlightIndex}
            onmousedown={() => selectTag(item.tag)}
          >
            {item.tag.name}
          </button>
        {/if}
      {/each}
      {#if showCreate}
        <button
          class="tag-input-option tag-input-create"
          class:tag-input-option-highlight={highlightIndex ===
            sortedSuggestions.length}
          onmousedown={() => createTag(query.trim())}
        >
          "{query.trim()}" を作成
        </button>
      {/if}
    </div>
  {/if}
</div>
