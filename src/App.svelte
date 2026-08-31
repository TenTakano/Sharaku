<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount } from "svelte";
  import AppSettingsView from "./lib/components/AppSettingsView.svelte";
  import BulkImportView from "./lib/components/BulkImportView.svelte";
  import ImportBanner from "./lib/components/ImportBanner.svelte";
  import ImportView from "./lib/components/ImportView.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import SetupView from "./lib/components/SetupView.svelte";
  import LibrarySettingsView from "./lib/components/LibrarySettingsView.svelte";
  import WorkGrid from "./lib/components/WorkGrid.svelte";
  import PlaylistView from "./lib/components/PlaylistView.svelte";
  import Toast from "./lib/components/Toast.svelte";
  import WorkViewer from "./lib/components/WorkViewer.svelte";
  import { initImportQueueListener } from "./lib/stores/importQueue.svelte";
  import { initBannerAutoClose } from "./lib/stores/bannerAutoClose.svelte";
  import { initTheme } from "./lib/stores/theme.svelte";
  import { addToast } from "./lib/stores/toast.svelte";
  import type {
    DropKind,
    ImportKind,
    Library,
    Playlist,
    Tag,
    TagSearchMode,
    ViewKind,
  } from "./lib/types";

  let reloadTrigger = $state(0);
  let playlistReloadTrigger = $state(0);
  let filterTags = $state<Tag[]>([]);
  let tagSearchMode = $state<TagSearchMode>("and");
  let currentView = $state<ViewKind>("library");
  let selectedWorkId = $state<number | null>(null);
  let workIds = $state<number[]>([]);
  let selectedPlaylist = $state<Playlist | null>(null);
  let viewerOrigin = $state<ViewKind>("library");
  let activeLibrary = $state<Library | null>(null);
  let libraryLoading = $state(true);
  let dragging = $state(false);
  let importSourcePath = $state<string | undefined>(undefined);
  let importSourceKind = $state<ImportKind>("folder");
  let bulkImportRootPath = $state<string | undefined>(undefined);
  let bulkImportDroppedPaths = $state<string[] | undefined>(undefined);
  let previousView = $state<ViewKind>("library");

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
    selectedPlaylist = null;
    reloadTrigger++;
  }

  function handleSelectWork(workId: number) {
    viewerOrigin = currentView === "playlist" ? "playlist" : "library";
    selectedWorkId = workId;
    currentView = "viewer";
  }

  function handleWorksLoaded(ids: number[]) {
    workIds = ids;
  }

  function handleNavigateWork(workId: number) {
    selectedWorkId = workId;
  }

  function resetToView(view: ViewKind) {
    currentView = view;
    selectedWorkId = null;
    importSourcePath = undefined;
    importSourceKind = "folder";
  }

  function handleBackToLibrary() {
    resetToView("library");
  }

  function handleViewerBack() {
    resetToView(viewerOrigin);
  }

  function handleSelectPlaylist(playlist: Playlist) {
    selectedPlaylist = playlist;
    currentView = "playlist";
  }

  function handlePlaylistRenamed(playlist: Playlist) {
    if (selectedPlaylist?.id === playlist.id) {
      selectedPlaylist = playlist;
    }
  }

  function handlePlaylistCreated() {
    playlistReloadTrigger++;
  }

  function handlePlaylistDeleted(id: number) {
    if (selectedPlaylist?.id === id) {
      selectedPlaylist = null;
      currentView = "library";
    }
  }

  function handleBackToSettings() {
    currentView = "settings";
    bulkImportRootPath = undefined;
  }

  function handleBulkImportBack() {
    if (bulkImportRootPath || bulkImportDroppedPaths) {
      bulkImportRootPath = undefined;
      bulkImportDroppedPaths = undefined;
      currentView = "library";
    } else {
      handleBackToSettings();
    }
  }

  async function handleDeleteLibrary() {
    activeLibrary = await invoke<Library | null>("get_active_library");
    if (activeLibrary) {
      currentView = "library";
      reloadTrigger++;
    }
  }

  function handleNavigateToAppSettings() {
    previousView = currentView;
    currentView = "app-settings";
  }

  function handleBackFromAppSettings() {
    currentView = activeLibrary ? previousView : "library";
  }

  function handleSidebarNavigate(view: ViewKind) {
    currentView = view;
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

  async function classifyDropPath(path: string): Promise<DropKind> {
    return invoke<DropKind>("classify_drop_path", { path });
  }

  onMount(() => {
    initTheme();
    initBannerAutoClose();
  });

  $effect(() => {
    loadActiveLibrary();
  });

  $effect(() => {
    const cleanup = initImportQueueListener(() => {
      reloadTrigger++;
    });
    return cleanup;
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
        if (paths.length > 1) {
          const unique = [...new Set(paths)];
          bulkImportDroppedPaths = unique;
          bulkImportRootPath = undefined;
          currentView = "bulk-import";
        } else if (paths.length === 1) {
          const path = paths[0];
          classifyDropPath(path)
            .then(async (kind) => {
              if (kind === "folder") {
                const hasSubs = await invoke<boolean>("has_image_subfolders", {
                  dir: path,
                });
                if (hasSubs) {
                  bulkImportRootPath = path;
                  bulkImportDroppedPaths = undefined;
                  currentView = "bulk-import";
                } else {
                  importSourcePath = path;
                  importSourceKind = "folder";
                  currentView = "import";
                }
              } else if (kind === "image") {
                importSourcePath = path;
                importSourceKind = "image";
                currentView = "import";
              } else {
                addToast("error", "対応していないファイル形式です");
              }
            })
            .catch((e) => {
              console.error("Drop classification failed:", e);
            });
        }
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

{#if currentView === "app-settings"}
  <div class="app-settings-layout">
    <main class="content-area">
      <div class="context-bar">
        <div class="context-bar-left">
          <button class="context-bar-back" onclick={handleBackFromAppSettings}>
            ←
          </button>
          <h1 class="context-bar-title">アプリ設定</h1>
        </div>
      </div>
      <AppSettingsView />
    </main>
  </div>
{:else if currentView === "viewer" && selectedWorkId !== null && activeLibrary}
  <WorkViewer
    workId={selectedWorkId}
    {workIds}
    libraryName={activeLibrary.name}
    onBack={handleViewerBack}
    onNavigateWork={handleNavigateWork}
  />
{:else if libraryLoading}
  <main class="container">
    <p class="library-loading">読み込み中...</p>
  </main>
{:else if currentView === "add-library"}
  <SetupView
    initialStep="mode-select"
    onComplete={handleLibrarySwitch}
    onCancel={handleBackToLibrary}
  />
{:else if !activeLibrary}
  <button class="setup-app-settings-link" onclick={handleNavigateToAppSettings}>
    &#9881; アプリ設定
  </button>
  <SetupView onComplete={handleLibrarySwitch} />
{:else}
  <div class="app-layout">
    <Sidebar
      {activeLibrary}
      {currentView}
      {reloadTrigger}
      {playlistReloadTrigger}
      onSwitchLibrary={handleLibrarySwitch}
      onNavigate={handleSidebarNavigate}
      onTagSelect={handleSidebarTagSelect}
      onNavigateToAppSettings={handleNavigateToAppSettings}
      selectedTagIds={filterTags.map((t) => t.id)}
      selectedPlaylistId={selectedPlaylist?.id ?? null}
      onSelectPlaylist={handleSelectPlaylist}
      onPlaylistRenamed={handlePlaylistRenamed}
      onPlaylistDeleted={handlePlaylistDeleted}
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
            <button class="context-bar-back" onclick={handleBulkImportBack}>
              ←
            </button>
            <h1 class="context-bar-title">
              一括取り込み: {activeLibrary.name}
            </h1>
          {:else if currentView === "playlist" && selectedPlaylist}
            <button class="context-bar-back" onclick={handleBackToLibrary}>
              ←
            </button>
            <h1 class="context-bar-title">{selectedPlaylist.name}</h1>
          {:else}
            <h1 class="context-bar-title">{activeLibrary.name}</h1>
          {/if}
        </div>
      </div>

      {#if currentView === "settings"}
        <LibrarySettingsView
          libraryId={activeLibrary.id}
          libraryName={activeLibrary.name}
          libraryPath={activeLibrary.path}
          onNavigate={handleSidebarNavigate}
          onDeleteLibrary={handleDeleteLibrary}
        />
      {:else if currentView === "import"}
        <ImportView
          initialSourcePath={importSourcePath}
          initialKind={importSourceKind}
          onBack={handleBackToLibrary}
        />
      {:else if currentView === "bulk-import"}
        <BulkImportView
          initialRootPath={bulkImportRootPath}
          initialDroppedPaths={bulkImportDroppedPaths}
          onBack={handleBulkImportBack}
        />
      {:else if currentView === "playlist" && selectedPlaylist}
        <PlaylistView
          playlistId={selectedPlaylist.id}
          onSelectWork={handleSelectWork}
          onWorksLoaded={handleWorksLoaded}
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
          onPlaylistCreated={handlePlaylistCreated}
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

<div class="notification-container">
  <ImportBanner />
  <Toast />
</div>
