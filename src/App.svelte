<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount } from "svelte";
  import AppSettingsView from "./lib/components/AppSettingsView.svelte";
  import BulkImportView from "./lib/components/BulkImportView.svelte";
  import HomeView from "./lib/components/HomeView.svelte";
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
  let currentView = $state<ViewKind>("home");
  let selectedWorkId = $state<number | null>(null);
  let workIds = $state<number[]>([]);
  let selectedPlaylist = $state<Playlist | null>(null);
  let viewerOrigin = $state<ViewKind>("home");
  let viewerStartPage = $state(0);
  let activeLibrary = $state<Library | null>(null);
  let libraryLoading = $state(true);
  let dragging = $state(false);
  let importSourcePath = $state<string | undefined>(undefined);
  let importSourceKind = $state<ImportKind>("folder");
  let bulkImportRootPath = $state<string | undefined>(undefined);
  let bulkImportDroppedPaths = $state<string[] | undefined>(undefined);
  let previousView = $state<ViewKind>("library");
  // Where "←" on the bulk import screen returns when it was opened from the
  // navigation or the library settings (drops return to the work grid).
  let bulkImportOrigin = $state<ViewKind>("settings");

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
    currentView = "home";
    selectedPlaylist = null;
    reloadTrigger++;
  }

  function handleSelectWork(workId: number) {
    viewerOrigin = currentView === "playlist" ? "playlist" : "library";
    viewerStartPage = 0;
    selectedWorkId = workId;
    currentView = "viewer";
  }

  function handleOpenFromHome(ids: number[], workId: number, page = 0) {
    workIds = ids;
    viewerOrigin = "home";
    viewerStartPage = page;
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
    } else if (bulkImportOrigin === "settings") {
      handleBackToSettings();
    } else {
      currentView = bulkImportOrigin;
    }
  }

  async function handleDeleteLibrary() {
    activeLibrary = await invoke<Library | null>("get_active_library");
    if (activeLibrary) {
      currentView = "home";
      reloadTrigger++;
    }
  }

  function handleNavigateToAppSettings() {
    previousView = currentView;
    currentView = "app-settings";
  }

  function handleBackFromAppSettings() {
    currentView = activeLibrary ? previousView : "home";
  }

  function handleSidebarNavigate(view: ViewKind) {
    if (view === "bulk-import") {
      bulkImportOrigin = currentView;
    }
    currentView = view;
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
      if (currentView !== "library" && currentView !== "home") return;
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
    initialPage={viewerStartPage}
    onBack={handleViewerBack}
    onNavigateWork={handleNavigateWork}
  />
{:else if libraryLoading}
  <main class="container">
    <p class="library-loading">読み込み中...</p>
  </main>
{:else if currentView === "add-library"}
  <SetupView
    initialStep="select"
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
      onNavigateToAppSettings={handleNavigateToAppSettings}
      selectedPlaylistId={selectedPlaylist?.id ?? null}
      onSelectPlaylist={handleSelectPlaylist}
      onPlaylistRenamed={handlePlaylistRenamed}
      onPlaylistDeleted={handlePlaylistDeleted}
    />
    <main class="content-area">
      {#if currentView !== "home"}
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
      {/if}

      {#if currentView === "home"}
        <HomeView
          {reloadTrigger}
          onOpenWork={handleOpenFromHome}
          onBrowseAll={() => (currentView = "library")}
        />
      {:else if currentView === "settings"}
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
          libraryPath={activeLibrary.path}
          onSelectWork={handleSelectWork}
          onWorksLoaded={handleWorksLoaded}
          onFilterTagsChange={(tags) => (filterTags = tags)}
          onTagSearchModeChange={(mode) => (tagSearchMode = mode)}
          onPlaylistCreated={handlePlaylistCreated}
        />
      {/if}

      {#if dragging && (currentView === "library" || currentView === "home")}
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
