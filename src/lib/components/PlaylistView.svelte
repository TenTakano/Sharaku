<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { DragDropProvider } from "@dnd-kit/svelte";
  import PlaylistItemRow from "./PlaylistItemRow.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import { addToast } from "../stores/toast.svelte";
  import { moveItem } from "../utils/reorder";
  import { createLatestRequestGuard } from "../utils/latestRequest";
  import type { PlaylistItem } from "../types";

  interface Props {
    playlistId: number;
    onSelectWork: (workId: number) => void;
    onWorksLoaded?: (workIds: number[]) => void;
  }

  let { playlistId, onSelectWork, onWorksLoaded }: Props = $props();

  interface SortEndpoint {
    id: string | number;
  }
  interface DragOverLikeEvent {
    operation: {
      source: SortEndpoint | null;
      target: SortEndpoint | null;
    };
  }
  interface DragEndLikeEvent extends DragOverLikeEvent {
    canceled: boolean;
  }

  let items = $state<PlaylistItem[]>([]);
  let removingItem = $state<PlaylistItem | null>(null);

  const itemsLoadGuard = createLatestRequestGuard();
  // persistOrderGuard alone only discards a stale *result* once it arrives; it does
  // not stop two persistOrder() calls from racing reorder_playlist_items on the
  // server while both are in flight, which could apply an older order last. Chaining
  // through persistOrderChain forces each call to wait for the previous one to finish
  // before it starts, so requests reach the server in the same order they were issued.
  const persistOrderGuard = createLatestRequestGuard();
  let persistOrderChain: Promise<void> = Promise.resolve();

  async function loadItems() {
    const currentId = itemsLoadGuard.next();
    try {
      const result = await invoke<PlaylistItem[]>("get_playlist_items", {
        playlistId,
      });
      if (itemsLoadGuard.isLatest(currentId)) {
        items = result;
        onWorksLoaded?.(items.map((i) => i.workId));
      }
    } catch {
      if (itemsLoadGuard.isLatest(currentId)) {
        items = [];
      }
    }
  }

  $effect(() => {
    void playlistId;
    loadItems();
  });

  function persistOrder() {
    const currentId = persistOrderGuard.next();
    const workIds = items.map((i) => i.workId);
    persistOrderChain = persistOrderChain.then(async () => {
      try {
        await invoke("reorder_playlist_items", { playlistId, workIds });
        if (persistOrderGuard.isLatest(currentId)) {
          onWorksLoaded?.(workIds);
        }
      } catch (e) {
        if (persistOrderGuard.isLatest(currentId)) {
          addToast("error", `並び替えに失敗しました: ${e}`);
          loadItems();
        }
      }
    });
  }

  function handleDragOver(event: DragOverLikeEvent) {
    const sourceId = event.operation.source?.id;
    const targetId = event.operation.target?.id;
    if (sourceId == null || targetId == null || sourceId === targetId) return;
    const from = items.find((i) => i.workId === sourceId);
    const to = items.find((i) => i.workId === targetId);
    if (!from || !to) return;
    items = moveItem(items, from, to);
  }

  function handleDragEnd(event: DragEndLikeEvent) {
    if (event.canceled) {
      loadItems();
      return;
    }
    persistOrder();
  }

  function openRemoveDialog(item: PlaylistItem) {
    removingItem = item;
  }

  async function handleRemove() {
    if (!removingItem) return;
    const { workId, title } = removingItem;
    try {
      await invoke("remove_item_from_playlist", { playlistId, workId });
      removingItem = null;
      addToast("success", `「${title}」をプレイリストから削除しました`);
      loadItems();
    } catch (e) {
      addToast("error", `削除に失敗しました: ${e}`);
    }
  }
</script>

<div class="playlist-view">
  {#if items.length > 0}
    <DragDropProvider onDragOver={handleDragOver} onDragEnd={handleDragEnd}>
      <div class="playlist-item-list">
        {#each items as item, index (item.workId)}
          <PlaylistItemRow
            {item}
            {index}
            {onSelectWork}
            onRemove={openRemoveDialog}
          />
        {/each}
      </div>
    </DragDropProvider>
  {:else}
    <div class="empty-state">
      <p>
        このプレイリストには作品がありません。作品を右クリックして「プレイリストに追加」から追加してください。
      </p>
    </div>
  {/if}
</div>

{#if removingItem}
  <ConfirmDialog
    title="プレイリストから削除"
    message="「{removingItem.title}」をプレイリストから削除しますか？"
    confirmLabel="削除"
    danger={true}
    onConfirm={handleRemove}
    onCancel={() => (removingItem = null)}
  />
{/if}
