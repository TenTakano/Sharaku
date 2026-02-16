<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import BulkImportView from "./lib/components/BulkImportView.svelte";
  import ImportView from "./lib/components/ImportView.svelte";
  import LibrarySwitcher from "./lib/components/LibrarySwitcher.svelte";
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

  $effect(() => {
    loadActiveLibrary();
  });
</script>

{#if currentView === "viewer" && selectedWorkId !== null}
  <WorkViewer
    workId={selectedWorkId}
    {workIds}
    onBack={handleBackToLibrary}
    onNavigateWork={handleNavigateWork}
  />
{:else if currentView === "settings"}
  <SettingsView
    onBack={handleBackToLibrary}
    libraryPath={activeLibrary?.path ?? null}
  />
{:else if currentView === "import"}
  <ImportView onBack={handleBackToLibrary} onImported={() => reloadTrigger++} />
{:else if currentView === "bulk-import"}
  <BulkImportView
    onBack={handleBackToLibrary}
    onImported={() => reloadTrigger++}
  />
{:else}
  <main class="container">
    <div class="app-header">
      <h1>Sharaku</h1>
      <LibrarySwitcher {activeLibrary} onSwitch={handleLibrarySwitch} />
      <button
        class="import-header-btn"
        onclick={() => (currentView = "import")}
        disabled={!activeLibrary}
      >
        + 取り込み
      </button>
      <button
        class="import-header-btn"
        onclick={() => (currentView = "bulk-import")}
        disabled={!activeLibrary}
      >
        一括取り込み
      </button>
      <button
        class="settings-btn"
        onclick={() => (currentView = "settings")}
        disabled={!activeLibrary}
        title="設定"
      >
        ⚙
      </button>
    </div>
    {#if libraryLoading}
      <p class="library-loading">読み込み中...</p>
    {:else if !activeLibrary}
      <div class="no-library-message">
        <p>ライブラリが選択されていません。</p>
        <p>上部のドロップダウンからライブラリを追加してください。</p>
      </div>
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
{/if}
