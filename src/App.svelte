<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import BulkImportView from "./lib/components/BulkImportView.svelte";
  import ImportView from "./lib/components/ImportView.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import SetupView from "./lib/components/SetupView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import WorkGrid from "./lib/components/WorkGrid.svelte";
  import WorkViewer from "./lib/components/WorkViewer.svelte";
  import type { Library, Tag, TagSearchMode } from "./lib/types";

  let reloadTrigger = $state(0);
  let filterTags = $state<Tag[]>([]);
  let tagSearchMode = $state<TagSearchMode>("and");
  let currentView = $state<
    "library" | "viewer" | "settings" | "import" | "bulk-import"
  >("library");
  let selectedWorkId = $state<number | null>(null);
  let workIds = $state<number[]>([]);
  let activeLibrary = $state<Library | null>(null);
  let libraryLoading = $state(true);
  let importDropdownOpen = $state(false);

  async function loadActiveLibrary() {
    try {
      activeLibrary = await invoke<Library | null>("get_active_library");
    } catch {
      activeLibrary = null;
    } finally {
      libraryLoading = false;
    }
  }

  function handleLibrarySwitch(library: Library) {
    activeLibrary = library;
    filterTags = [];
    currentView = "library";
    reloadTrigger++;
  }

  function handleSelectWork(workId: number) {
    selectedWorkId = workId;
    currentView = "viewer";
  }

  function handleWorksLoaded(ids: number[]) {
    workIds = ids;
  }

  function handleNavigateWork(workId: number) {
    selectedWorkId = workId;
  }

  function handleBackToLibrary() {
    currentView = "library";
    selectedWorkId = null;
  }

  function handleSidebarNavigate(view: string) {
    currentView = view as typeof currentView;
    importDropdownOpen = false;
  }

  function handleSidebarTagSelect(tag: Tag) {
    if (currentView !== "library") {
      currentView = "library";
    }
    const exists = filterTags.some((t) => t.id === tag.id);
    if (exists) {
      filterTags = filterTags.filter((t) => t.id !== tag.id);
    } else {
      filterTags = [...filterTags, tag];
    }
  }

  function toggleImportDropdown() {
    importDropdownOpen = !importDropdownOpen;
  }

  function closeImportDropdown() {
    importDropdownOpen = false;
  }

  $effect(() => {
    loadActiveLibrary();
  });
</script>

{#if currentView === "viewer" && selectedWorkId !== null && activeLibrary}
  <WorkViewer
    workId={selectedWorkId}
    {workIds}
    libraryName={activeLibrary.name}
    onBack={handleBackToLibrary}
    onNavigateWork={handleNavigateWork}
  />
{:else if libraryLoading}
  <main class="container">
    <p class="library-loading">読み込み中...</p>
  </main>
{:else if !activeLibrary}
  <SetupView onComplete={handleLibrarySwitch} />
{:else}
  <div class="app-layout">
    <Sidebar
      {activeLibrary}
      {currentView}
      onSwitchLibrary={handleLibrarySwitch}
      onNavigate={handleSidebarNavigate}
      onTagSelect={handleSidebarTagSelect}
      selectedTagIds={filterTags.map((t) => t.id)}
    />
    <main class="content-area">
      <div class="context-bar">
        <div class="context-bar-left">
          {#if currentView === "settings"}
            <button class="context-bar-back" onclick={handleBackToLibrary}>
              ←
            </button>
            <h1 class="context-bar-title">設定: {activeLibrary.name}</h1>
          {:else if currentView === "import" || currentView === "bulk-import"}
            <button class="context-bar-back" onclick={handleBackToLibrary}>
              ←
            </button>
            <h1 class="context-bar-title">取り込み: {activeLibrary.name}</h1>
          {:else}
            <h1 class="context-bar-title">{activeLibrary.name}</h1>
          {/if}
        </div>
        {#if currentView === "library"}
          <div class="context-bar-actions">
            <div class="import-dropdown-wrapper">
              <button
                class="context-bar-import-btn"
                onclick={toggleImportDropdown}
              >
                + 取り込み ▾
              </button>
              {#if importDropdownOpen}
                <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
                <div
                  class="import-dropdown-overlay"
                  onclick={closeImportDropdown}
                ></div>
                <div class="import-dropdown-menu">
                  <button
                    class="import-dropdown-item"
                    onclick={() => {
                      currentView = "import";
                      closeImportDropdown();
                    }}
                  >
                    単体取り込み
                  </button>
                  <button
                    class="import-dropdown-item"
                    onclick={() => {
                      currentView = "bulk-import";
                      closeImportDropdown();
                    }}
                  >
                    一括取り込み
                  </button>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      {#if currentView === "settings"}
        <SettingsView libraryPath={activeLibrary.path} />
      {:else if currentView === "import"}
        <ImportView
          onBack={handleBackToLibrary}
          onImported={() => reloadTrigger++}
        />
      {:else if currentView === "bulk-import"}
        <BulkImportView
          onBack={handleBackToLibrary}
          onImported={() => reloadTrigger++}
        />
      {:else}
        <WorkGrid
          {reloadTrigger}
          {filterTags}
          {tagSearchMode}
          onSelectWork={handleSelectWork}
          onWorksLoaded={handleWorksLoaded}
          onFilterTagsChange={(tags) => (filterTags = tags)}
          onTagSearchModeChange={(mode) => (tagSearchMode = mode)}
        />
      {/if}
    </main>
  </div>
{/if}
