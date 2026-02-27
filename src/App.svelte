<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import BulkImportView from "./lib/components/BulkImportView.svelte";
  import ImportView from "./lib/components/ImportView.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import SetupView from "./lib/components/SetupView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import WorkGrid from "./lib/components/WorkGrid.svelte";
  import WorkViewer from "./lib/components/WorkViewer.svelte";
  import type {
    Library,
    Tag,
    TagSearchMode,
    UnregisteredEntry,
  } from "./lib/types";

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
  let dragging = $state(false);
  let importSourcePath = $state<string | undefined>(undefined);
  let pendingBulkImportEntries = $state<UnregisteredEntry[] | undefined>(
    undefined,
  );

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
    importSourcePath = undefined;
  }

  function handleBackToSettings() {
    currentView = "settings";
    pendingBulkImportEntries = undefined;
  }

  function handleImportUnregistered(entries: UnregisteredEntry[]) {
    pendingBulkImportEntries = entries;
    currentView = "bulk-import";
  }

  function handleSidebarNavigate(view: string) {
    currentView = view as typeof currentView;
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

  async function resolveFolderPath(path: string): Promise<string> {
    return invoke<string>("resolve_drop_path", { path });
  }

  $effect(() => {
    loadActiveLibrary();
  });

  $effect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (currentView !== "library") return;
      if (event.payload.type === "enter") {
        dragging = true;
      } else if (event.payload.type === "leave") {
        dragging = false;
      } else if (event.payload.type === "drop") {
        dragging = false;
        const paths = event.payload.paths;
        if (paths.length > 0) {
          resolveFolderPath(paths[0])
            .then((folderPath) => {
              importSourcePath = folderPath;
              currentView = "import";
            })
            .catch((e) => {
              console.error("Drop path resolution failed:", e);
            });
        }
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
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
          {:else if currentView === "import"}
            <button class="context-bar-back" onclick={handleBackToLibrary}>
              ←
            </button>
            <h1 class="context-bar-title">取り込み: {activeLibrary.name}</h1>
          {:else if currentView === "bulk-import"}
            <button class="context-bar-back" onclick={handleBackToSettings}>
              ←
            </button>
            <h1 class="context-bar-title">
              一括取り込み: {activeLibrary.name}
            </h1>
          {:else}
            <h1 class="context-bar-title">{activeLibrary.name}</h1>
          {/if}
        </div>
      </div>

      {#if currentView === "settings"}
        <SettingsView
          libraryPath={activeLibrary.path}
          onNavigate={handleSidebarNavigate}
          onImportUnregistered={handleImportUnregistered}
        />
      {:else if currentView === "import"}
        <ImportView
          initialSourcePath={importSourcePath}
          onBack={handleBackToLibrary}
          onImported={() => reloadTrigger++}
        />
      {:else if currentView === "bulk-import"}
        <BulkImportView
          initialEntries={pendingBulkImportEntries}
          onBack={handleBackToSettings}
          onImported={() => {
            reloadTrigger++;
            pendingBulkImportEntries = undefined;
          }}
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

      {#if dragging && currentView === "library"}
        <div class="drop-overlay">
          <div class="drop-overlay-content">
            ここにフォルダをドロップして取り込み
          </div>
        </div>
      {/if}
    </main>
  </div>
{/if}
