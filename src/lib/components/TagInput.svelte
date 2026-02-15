<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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
  let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let inputEl = $state<HTMLInputElement | null>(null);

  let searchGeneration = 0;

  async function search(q: string) {
    if (!q.trim()) {
      suggestions = [];
      open = false;
      return;
    }
    const gen = ++searchGeneration;
    try {
      const results: Tag[] = await invoke("search_tags", {
        query: q.trim(),
        category: null,
      });
      if (gen !== searchGeneration) return;
      suggestions = results.filter((t) => !excludeTagIds.includes(t.id));
      open = true;
      highlightIndex = -1;
    } catch {
      if (gen !== searchGeneration) return;
      suggestions = [];
    }
  }

  function handleInput(e: Event) {
    query = (e.target as HTMLInputElement).value;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => search(query), 200);
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

  let hasExactMatch = $derived(
    suggestions.some(
      (t) => t.name.toLowerCase() === query.trim().toLowerCase(),
    ),
  );
  let showCreate = $derived(
    open && query.trim() && !hasExactMatch && !!onCreateTag,
  );
  let totalOptions = $derived(suggestions.length + (showCreate ? 1 : 0));

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
        if (highlightIndex >= 0 && highlightIndex < suggestions.length) {
          selectTag(suggestions[highlightIndex]);
        } else if (showCreate && highlightIndex === suggestions.length) {
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
      {#each suggestions as tag, i (tag.id)}
        <button
          class="tag-input-option"
          class:tag-input-option-highlight={i === highlightIndex}
          onmousedown={() => selectTag(tag)}
        >
          {tag.name}
        </button>
      {/each}
      {#if showCreate}
        <button
          class="tag-input-option tag-input-create"
          class:tag-input-option-highlight={highlightIndex ===
            suggestions.length}
          onmousedown={() => createTag(query.trim())}
        >
          "{query.trim()}" を作成
        </button>
      {/if}
    </div>
  {/if}
</div>
