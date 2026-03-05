<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import type { Library, Tag } from "../types";

  interface Props {
    activeLibrary: Library;
    currentView: string;
    reloadTrigger: number;
    onSwitchLibrary: (library: Library) => void;
    onNavigate: (view: string) => void;
    onTagSelect: (tag: Tag) => void;
    onNavigateToAppSettings: () => void;
    selectedTagIds: number[];
  }

  let {
    activeLibrary,
    currentView,
    reloadTrigger,
    onSwitchLibrary,
    onNavigate,
    onTagSelect,
    onNavigateToAppSettings,
    selectedTagIds,
  }: Props = $props();

  let libraries = $state<Library[]>([]);
  let tags = $state<Tag[]>([]);
  let expandedCategories = new SvelteSet<string>();

  interface TagsByCategory {
    category: string | null;
    displayName: string;
    tags: Tag[];
  }

  let tagsByCategory = $derived.by(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- local variable in $derived, not reactive state
    const map = new Map<string | null, Tag[]>();
    for (const tag of tags) {
      const key = tag.category ?? null;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(tag);
    }
    const result: TagsByCategory[] = [];
    for (const [category, categoryTags] of map) {
      result.push({
        category,
        displayName: category ?? "other",
        tags: categoryTags,
      });
    }
    return result;
  });

  async function loadLibraries() {
    try {
      libraries = await invoke<Library[]>("list_libraries");
    } catch {
      libraries = [];
    }
  }

  let tagLoadId = 0;

  async function loadTags() {
    const currentId = ++tagLoadId;
    try {
      const result = await invoke<Tag[]>("list_tags");
      if (currentId === tagLoadId) {
        tags = result;
      }
    } catch {
      if (currentId === tagLoadId) {
        tags = [];
      }
    }
  }

  async function selectLibrary(lib: Library) {
    if (lib.id === activeLibrary?.id) return;
    try {
      await invoke("switch_library", { id: lib.id });
      onSwitchLibrary(lib);
    } catch (e) {
      console.error("ライブラリ切り替え失敗:", e);
    }
  }

  function toggleCategory(category: string | null) {
    const key = category ?? "__null__";
    if (expandedCategories.has(key)) {
      expandedCategories.delete(key);
    } else {
      expandedCategories.add(key);
    }
  }

  function isCategoryExpanded(category: string | null): boolean {
    return expandedCategories.has(category ?? "__null__");
  }

  function isTagSelected(tagId: number): boolean {
    return selectedTagIds.includes(tagId);
  }

  $effect(() => {
    void reloadTrigger;
    loadLibraries();
  });

  $effect(() => {
    void activeLibrary;
    loadTags();
  });
</script>

<aside class="sidebar">
  <div class="sidebar-logo">SHARAKU</div>

  <div class="sidebar-section">
    <div class="sidebar-section-header">Libraries</div>
    <div class="sidebar-library-list">
      {#each libraries as lib (lib.id)}
        <button
          class="sidebar-library-item"
          class:active={lib.id === activeLibrary?.id}
          onclick={() => selectLibrary(lib)}
        >
          <span class="sidebar-library-name">{lib.name}</span>
          {#if lib.id === activeLibrary?.id}
            <span
              class="sidebar-library-settings"
              class:settings-active={currentView === "settings"}
              role="button"
              tabindex="-1"
              onclick={(e: MouseEvent) => {
                e.stopPropagation();
                onNavigate("settings");
              }}
              onkeydown={(e: KeyboardEvent) => {
                if (e.key === "Enter") {
                  e.stopPropagation();
                  onNavigate("settings");
                }
              }}
              title="ライブラリ設定"
            >
              &#9881;
            </span>
          {/if}
        </button>
      {/each}
    </div>
    <button
      class="sidebar-add-library"
      onclick={() => onNavigate("add-library")}
    >
      + ライブラリを追加
    </button>
  </div>

  {#if tags.length > 0}
    <div class="sidebar-section sidebar-tags-section">
      <div class="sidebar-section-header">Tags</div>
      <div class="sidebar-tag-browser">
        {#each tagsByCategory as group (group.displayName)}
          <div class="sidebar-tag-category">
            <button
              class="sidebar-tag-category-header"
              onclick={() => toggleCategory(group.category)}
            >
              <span class="sidebar-tag-category-arrow">
                {isCategoryExpanded(group.category) ? "\u25BC" : "\u25B6"}
              </span>
              {group.displayName}
              <span class="sidebar-tag-category-count"
                >({group.tags.length})</span
              >
            </button>
            {#if isCategoryExpanded(group.category)}
              <div class="sidebar-tag-list">
                {#each group.tags as tag (tag.id)}
                  <button
                    class="sidebar-tag-item"
                    class:selected={isTagSelected(tag.id)}
                    data-category={tag.category}
                    onclick={() => onTagSelect(tag)}
                  >
                    {tag.name}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <div class="sidebar-footer">
    <button
      class="sidebar-app-settings"
      class:active={currentView === "app-settings"}
      onclick={onNavigateToAppSettings}
    >
      &#9881; アプリ設定
    </button>
  </div>
</aside>
