<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createLatestRequestGuard } from "../utils/latestRequest";
  import { focusOnMount } from "../utils/focusOnMount";
  import { addToast } from "../stores/toast.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import type { Library, Playlist, ViewKind } from "../types";

  interface Props {
    activeLibrary: Library;
    currentView: ViewKind;
    reloadTrigger: number;
    playlistReloadTrigger: number;
    onSwitchLibrary: (library: Library) => void;
    onNavigate: (view: ViewKind) => void;
    onNavigateToAppSettings: () => void;
    selectedPlaylistId: number | null;
    onSelectPlaylist: (playlist: Playlist) => void;
    onPlaylistRenamed: (playlist: Playlist) => void;
    onPlaylistDeleted: (id: number) => void;
  }

  let {
    activeLibrary,
    currentView,
    reloadTrigger,
    playlistReloadTrigger,
    onSwitchLibrary,
    onNavigate,
    onNavigateToAppSettings,
    selectedPlaylistId,
    onSelectPlaylist,
    onPlaylistRenamed,
    onPlaylistDeleted,
  }: Props = $props();

  let libraries = $state<Library[]>([]);
  let libraryMenuOpen = $state(false);
  let footerEl = $state<HTMLDivElement | null>(null);
  let libraryMenuEl = $state<HTMLDivElement | null>(null);
  let librarySwitcherEl = $state<HTMLButtonElement | null>(null);

  function handleWindowPointerDown(e: PointerEvent) {
    if (libraryMenuOpen && !footerEl?.contains(e.target as Node)) {
      libraryMenuOpen = false;
    }
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (libraryMenuOpen && e.key === "Escape") {
      libraryMenuOpen = false;
      librarySwitcherEl?.focus();
    }
  }

  function libraryMenuItems(): HTMLElement[] {
    return Array.from(
      libraryMenuEl?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [],
    );
  }

  // Menu keyboard pattern: ↑/↓ wrap around, Home/End jump, Tab leaves and closes.
  function handleLibraryMenuKeydown(e: KeyboardEvent) {
    if (e.key === "Tab") {
      libraryMenuOpen = false;
      return;
    }
    const items = libraryMenuItems();
    if (items.length === 0) return;
    const current = items.indexOf(document.activeElement as HTMLElement);
    let next: number;
    if (e.key === "ArrowDown") next = (current + 1) % items.length;
    else if (e.key === "ArrowUp")
      next = (current - 1 + items.length) % items.length;
    else if (e.key === "Home") next = 0;
    else if (e.key === "End") next = items.length - 1;
    else return;
    e.preventDefault();
    items[next].focus();
  }

  // Opening the menu moves focus into it, onto the active library.
  $effect(() => {
    if (!libraryMenuOpen || !libraryMenuEl) return;
    const items = libraryMenuItems();
    (items.find((el) => el.classList.contains("active")) ?? items[0])?.focus();
  });

  let playlists = $state<Playlist[]>([]);
  let creatingPlaylist = $state(false);
  let newPlaylistName = $state("");
  let renamingPlaylistId = $state<number | null>(null);
  let renamePlaylistName = $state("");
  let playlistContextMenu = $state<{
    x: number;
    y: number;
    playlist: Playlist;
  } | null>(null);
  let deletingPlaylist = $state<Playlist | null>(null);

  async function loadLibraries() {
    try {
      libraries = await invoke<Library[]>("list_libraries");
    } catch {
      libraries = [];
    }
  }

  function navigateFromLibraryMenu(view: ViewKind) {
    libraryMenuOpen = false;
    onNavigate(view);
  }

  async function selectLibrary(lib: Library) {
    libraryMenuOpen = false;
    if (lib.id === activeLibrary?.id) return;
    try {
      await invoke("switch_library", { id: lib.id });
      onSwitchLibrary(lib);
    } catch (e) {
      console.error("ライブラリ切り替え失敗:", e);
    }
  }

  const playlistLoadGuard = createLatestRequestGuard();

  async function loadPlaylists() {
    const currentId = playlistLoadGuard.next();
    try {
      const result = await invoke<Playlist[]>("list_playlists");
      if (playlistLoadGuard.isLatest(currentId)) {
        playlists = result ?? [];
      }
    } catch {
      if (playlistLoadGuard.isLatest(currentId)) {
        playlists = [];
      }
    }
  }

  function startCreatingPlaylist() {
    creatingPlaylist = true;
    newPlaylistName = "";
  }

  function cancelCreatingPlaylist() {
    creatingPlaylist = false;
    newPlaylistName = "";
  }

  async function submitCreatePlaylist() {
    const name = newPlaylistName.trim();
    if (!name) {
      cancelCreatingPlaylist();
      return;
    }
    try {
      const playlist = await invoke<Playlist>("create_playlist", { name });
      cancelCreatingPlaylist();
      await loadPlaylists();
      onSelectPlaylist(playlist);
    } catch (e) {
      addToast("error", `プレイリストの作成に失敗しました: ${e}`);
    }
  }

  function openPlaylistContextMenu(playlist: Playlist, e: MouseEvent) {
    e.preventDefault();
    playlistContextMenu = { x: e.clientX, y: e.clientY, playlist };
  }

  function handlePlaylistNameInputKeydown(
    e: KeyboardEvent,
    onEscape: () => void,
  ) {
    const input = e.currentTarget as HTMLInputElement;
    if (e.key === "Enter") {
      input.blur();
    } else if (e.key === "Escape") {
      onEscape();
      input.blur();
    }
  }

  function handleCreatePlaylistKeydown(e: KeyboardEvent) {
    handlePlaylistNameInputKeydown(e, () => {
      newPlaylistName = "";
    });
  }

  function handleRenamePlaylistKeydown(e: KeyboardEvent, playlist: Playlist) {
    handlePlaylistNameInputKeydown(e, () => {
      renamePlaylistName = playlist.name;
    });
  }

  function startRenamingPlaylist(playlist: Playlist) {
    playlistContextMenu = null;
    renamingPlaylistId = playlist.id;
    renamePlaylistName = playlist.name;
  }

  function cancelRenamingPlaylist() {
    renamingPlaylistId = null;
    renamePlaylistName = "";
  }

  async function submitRenamePlaylist(playlist: Playlist) {
    const name = renamePlaylistName.trim();
    if (!name || name === playlist.name) {
      cancelRenamingPlaylist();
      return;
    }
    try {
      await invoke("rename_playlist", { id: playlist.id, name });
      cancelRenamingPlaylist();
      await loadPlaylists();
      onPlaylistRenamed({ id: playlist.id, name });
    } catch (e) {
      addToast("error", `プレイリスト名の変更に失敗しました: ${e}`);
    }
  }

  function openDeletePlaylistDialog(playlist: Playlist) {
    playlistContextMenu = null;
    deletingPlaylist = playlist;
  }

  async function confirmDeletePlaylist() {
    if (!deletingPlaylist) return;
    const { id } = deletingPlaylist;
    try {
      await invoke("delete_playlist", { id });
      deletingPlaylist = null;
      await loadPlaylists();
      onPlaylistDeleted(id);
    } catch (e) {
      addToast("error", `プレイリストの削除に失敗しました: ${e}`);
    }
  }

  $effect(() => {
    void reloadTrigger;
    loadLibraries();
  });

  $effect(() => {
    void reloadTrigger;
    void playlistReloadTrigger;
    void activeLibrary;
    loadPlaylists();
  });
</script>

<svelte:window
  onpointerdown={handleWindowPointerDown}
  onkeydown={handleWindowKeydown}
/>

<aside class="sidebar">
  <div class="sidebar-logo">
    <span class="sidebar-logo-mark">写楽</span>
    <span class="sidebar-logo-text">SHARAKU</span>
  </div>

  <nav class="sidebar-nav" aria-label="メインナビゲーション">
    <button
      class="sidebar-nav-item"
      class:active={currentView === "home"}
      aria-current={currentView === "home" ? "page" : undefined}
      onclick={() => onNavigate("home")}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" class="sidebar-nav-icon"
        ><path d="M3 11l9-7 9 7"></path><path d="M5 10v10h14V10"></path></svg
      >
      ホーム
    </button>
    <button
      class="sidebar-nav-item"
      class:active={currentView === "library"}
      aria-current={currentView === "library" ? "page" : undefined}
      onclick={() => onNavigate("library")}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" class="sidebar-nav-icon"
        ><circle cx="11" cy="11" r="7"></circle><path d="M20 20l-4-4"
        ></path></svg
      >
      ブラウズ
    </button>
    <div
      class="sidebar-nav-item sidebar-nav-group"
      class:active={currentView === "playlist"}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" class="sidebar-nav-icon"
        ><rect x="3" y="5" width="18" height="14" rx="2"></rect><path
          d="M3 9h18"
        ></path></svg
      >
      コレクション
    </div>
    <div class="sidebar-playlist-list">
      {#each playlists as playlist (playlist.id)}
        {#if renamingPlaylistId === playlist.id}
          <input
            class="sidebar-playlist-rename-input"
            bind:value={renamePlaylistName}
            onkeydown={(e: KeyboardEvent) =>
              handleRenamePlaylistKeydown(e, playlist)}
            onblur={() => submitRenamePlaylist(playlist)}
            use:focusOnMount
          />
        {:else}
          <button
            class="sidebar-playlist-item"
            class:active={currentView === "playlist" &&
              playlist.id === selectedPlaylistId}
            onclick={() => onSelectPlaylist(playlist)}
            oncontextmenu={(e) => openPlaylistContextMenu(playlist, e)}
          >
            <span class="sidebar-playlist-name">{playlist.name}</span>
          </button>
        {/if}
      {/each}
      {#if creatingPlaylist}
        <input
          class="sidebar-playlist-create-input"
          placeholder="プレイリスト名"
          bind:value={newPlaylistName}
          onkeydown={handleCreatePlaylistKeydown}
          onblur={submitCreatePlaylist}
          use:focusOnMount
        />
      {:else}
        <button class="sidebar-add-playlist" onclick={startCreatingPlaylist}>
          + 新規プレイリスト作成
        </button>
      {/if}
    </div>
  </nav>

  <div class="sidebar-footer" bind:this={footerEl}>
    {#if libraryMenuOpen}
      <div
        class="sidebar-library-menu"
        role="menu"
        aria-label="ライブラリ"
        tabindex="-1"
        bind:this={libraryMenuEl}
        onkeydown={handleLibraryMenuKeydown}
      >
        {#each libraries as lib (lib.id)}
          <button
            class="sidebar-library-item"
            class:active={lib.id === activeLibrary?.id}
            role="menuitem"
            aria-current={lib.id === activeLibrary?.id ? "true" : undefined}
            onclick={() => selectLibrary(lib)}
          >
            {lib.name}
          </button>
        {/each}
        <div class="sidebar-library-menu-separator" role="separator"></div>
        <button
          class="sidebar-library-item"
          class:active={currentView === "settings"}
          role="menuitem"
          onclick={() => navigateFromLibraryMenu("settings")}
        >
          ライブラリ設定
        </button>
        <button
          class="sidebar-library-item"
          role="menuitem"
          onclick={() => navigateFromLibraryMenu("add-library")}
        >
          + ライブラリを追加
        </button>
      </div>
    {/if}
    <button
      class="sidebar-library-switcher"
      bind:this={librarySwitcherEl}
      aria-haspopup="menu"
      aria-expanded={libraryMenuOpen}
      onclick={() => (libraryMenuOpen = !libraryMenuOpen)}
    >
      <span class="sidebar-library-switcher-text">
        <span class="sidebar-library-switcher-label">ライブラリ</span>
        <span class="sidebar-library-switcher-name">{activeLibrary.name}</span>
      </span>
      <svg aria-hidden="true" viewBox="0 0 24 24" class="sidebar-nav-icon"
        ><path d="M6 15l6-6 6 6"></path></svg
      >
    </button>
    <div class="sidebar-footer-row">
      <button
        class="sidebar-footer-button sidebar-import"
        class:active={currentView === "bulk-import"}
        onclick={() => onNavigate("bulk-import")}
      >
        取り込む
      </button>
      <button
        class="sidebar-footer-button sidebar-app-settings"
        class:active={currentView === "app-settings"}
        aria-label="アプリ設定"
        title="アプリ設定"
        onclick={onNavigateToAppSettings}
      >
        &#9881;
      </button>
    </div>
  </div>
</aside>

{#if playlistContextMenu}
  <ContextMenu
    x={playlistContextMenu.x}
    y={playlistContextMenu.y}
    items={[
      {
        label: "名前を変更",
        action: () => startRenamingPlaylist(playlistContextMenu!.playlist),
      },
      {
        label: "削除",
        action: () => openDeletePlaylistDialog(playlistContextMenu!.playlist),
      },
    ]}
    onClose={() => (playlistContextMenu = null)}
  />
{/if}

{#if deletingPlaylist}
  <ConfirmDialog
    title="プレイリストの削除"
    message="「{deletingPlaylist.name}」を削除しますか？この操作は取り消せません。"
    confirmLabel="削除"
    danger={true}
    onConfirm={confirmDeletePlaylist}
    onCancel={() => (deletingPlaylist = null)}
  />
{/if}
