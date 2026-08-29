<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { addToast } from "../stores/toast.svelte";
  import { focusOnMount } from "../utils/focusOnMount";
  import { createLatestRequestGuard } from "../utils/latestRequest";
  import type { Playlist } from "../types";

  interface Props {
    workId: number;
    onClose: () => void;
    onPlaylistCreated?: () => void;
  }

  let { workId, onClose, onPlaylistCreated }: Props = $props();

  let playlists = $state<Playlist[]>([]);
  let creatingPlaylist = $state(false);
  let newPlaylistName = $state("");

  const playlistLoadGuard = createLatestRequestGuard();

  async function loadPlaylists() {
    const currentId = playlistLoadGuard.next();
    try {
      const result = await invoke<Playlist[]>("list_playlists");
      if (playlistLoadGuard.isLatest(currentId)) {
        playlists = result;
      }
    } catch {
      if (playlistLoadGuard.isLatest(currentId)) {
        playlists = [];
      }
    }
  }

  $effect(() => {
    loadPlaylists();
  });

  async function addToPlaylist(playlist: Playlist) {
    try {
      await invoke("add_item_to_playlist", { playlistId: playlist.id, workId });
      addToast("success", `「${playlist.name}」に追加しました`);
      onClose();
    } catch (e) {
      addToast("error", `追加に失敗しました: ${e}`);
    }
  }

  async function submitCreateAndAdd() {
    const name = newPlaylistName.trim();
    if (!name) return;
    let playlist: Playlist;
    try {
      playlist = await invoke<Playlist>("create_playlist", { name });
    } catch (e) {
      addToast("error", `プレイリストの作成に失敗しました: ${e}`);
      return;
    }
    onPlaylistCreated?.();
    creatingPlaylist = false;
    newPlaylistName = "";
    await loadPlaylists();
    await addToPlaylist(playlist);
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="add-to-playlist-overlay" onmousedown={handleOverlayClick}>
  <div class="add-to-playlist-dialog">
    <h3 class="add-to-playlist-title">プレイリストに追加</h3>
    {#if playlists.length > 0}
      <div class="add-to-playlist-list">
        {#each playlists as playlist (playlist.id)}
          <button
            class="add-to-playlist-item"
            onclick={() => addToPlaylist(playlist)}
          >
            {playlist.name}
          </button>
        {/each}
      </div>
    {:else}
      <p class="add-to-playlist-empty">プレイリストがありません</p>
    {/if}

    {#if creatingPlaylist}
      <input
        class="add-to-playlist-create-input"
        placeholder="新しいプレイリスト名"
        bind:value={newPlaylistName}
        onkeydown={(e: KeyboardEvent) => {
          if (e.key === "Enter") submitCreateAndAdd();
        }}
        use:focusOnMount
      />
    {:else}
      <button
        class="add-to-playlist-create-btn"
        onclick={() => (creatingPlaylist = true)}
      >
        + 新規プレイリストを作成して追加
      </button>
    {/if}

    <div class="add-to-playlist-actions">
      <button class="add-to-playlist-cancel" onclick={onClose}>
        キャンセル
      </button>
    </div>
  </div>
</div>
