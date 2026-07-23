<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import { createLatestRequestGuard } from "../utils/latestRequest";
  import { groupTagsByCategory } from "../utils/tagGrouping";
  import { focusOnMount } from "../utils/focusOnMount";
  import { addToast } from "../stores/toast.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import type { Library, Playlist, Tag, ViewKind } from "../types";

  interface Props {
    activeLibrary: Library;
    currentView: ViewKind;
    reloadTrigger: number;
    playlistReloadTrigger: number;
    onSwitchLibrary: (library: Library) => void;
    onNavigate: (view: ViewKind) => void;
    onTagSelect: (tag: Tag) => void;
    onNavigateToAppSettings: () => void;
    selectedTagIds: number[];
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
    onTagSelect,
    onNavigateToAppSettings,
    selectedTagIds,
    selectedPlaylistId,
    onSelectPlaylist,
    onPlaylistRenamed,
    onPlaylistDeleted,
  }: Props = $props();

  let libraries = $state<Library[]>([]);
  let tags = $state<Tag[]>([]);
  let expandedCategories = new SvelteSet<string>();

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

  let tagsByCategory = $derived(groupTagsByCategory(tags));

  async function loadLibraries() {
    try {
      libraries = await invoke<Library[]>("list_libraries");
    } catch {
      libraries = [];
    }
  }

  const tagLoadGuard = createLatestRequestGuard();

  async function loadTags() {
    const currentId = tagLoadGuard.next();
    try {
      const result = await invoke<Tag[]>("list_tags");
      if (tagLoadGuard.isLatest(currentId)) {
        tags = result;
      }
    } catch {
      if (tagLoadGuard.isLatest(currentId)) {
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

  $effect(() => {
    void reloadTrigger;
    void playlistReloadTrigger;
    void activeLibrary;
    loadPlaylists();
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

  <div class="sidebar-section">
    <div class="sidebar-section-header">Playlists</div>
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
    </div>
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
