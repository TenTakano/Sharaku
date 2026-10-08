<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { VList } from "virtua/svelte";
  import WorkCard from "./WorkCard.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import EditWorkDialog from "./EditWorkDialog.svelte";
  import AddToPlaylistDialog from "./AddToPlaylistDialog.svelte";
  import { addToast } from "../stores/toast.svelte";
  import { clearThumbnailCache } from "../thumbnailCache";
  import type {
    WorkSummary,
    WorkDetail,
    SortField,
    SortOrder,
    Tag,
    TagSearchMode,
    AppSettings,
    DeleteFileAction,
  } from "../types";
  import TagInput from "./TagInput.svelte";

  interface Props {
    reloadTrigger: number;
    filterTags: Tag[];
    tagSearchMode: TagSearchMode;
    libraryPath?: string | null;
    onSelectWork: (workId: number) => void;
    onWorksLoaded?: (workIds: number[]) => void;
    onFilterTagsChange: (tags: Tag[]) => void;
    onTagSearchModeChange: (mode: TagSearchMode) => void;
    onPlaylistCreated?: () => void;
  }

  let {
    reloadTrigger,
    filterTags,
    tagSearchMode,
    libraryPath = null,
    onSelectWork,
    onWorksLoaded,
    onFilterTagsChange,
    onTagSearchModeChange,
    onPlaylistCreated,
  }: Props = $props();

  let works = $state<WorkSummary[]>([]);
  let sortField = $state<SortField>("created_at");
  let sortOrder = $state<SortOrder>("desc");
  let containerWidth = $state(0);
  let contextMenu = $state<{ x: number; y: number; workId: number } | null>(
    null,
  );
  let editingWork = $state<WorkDetail | null>(null);
  let deletingWork = $state<{
    id: number;
    title: string;
    path: string;
    workType: string;
  } | null>(null);
  let addingToPlaylistWorkId = $state<number | null>(null);
  let deleteFileAction = $state<DeleteFileAction>("ask");
  let selectedFileAction = $state<DeleteFileAction | "none">("none");

  const CARD_WIDTH = 180;
  const GAP = 16;

  let columnCount = $derived(
    Math.max(1, Math.floor((containerWidth + GAP) / (CARD_WIDTH + GAP))),
  );

  let rows = $derived.by(() => {
    const result: WorkSummary[][] = [];
    for (let i = 0; i < works.length; i += columnCount) {
      result.push(works.slice(i, i + columnCount));
    }
    return result;
  });

  function sortWorksClientSide(
    list: WorkSummary[],
    field: SortField,
    order: SortOrder,
  ): WorkSummary[] {
    return [...list].sort((a, b) => {
      let cmp: number;
      if (field === "title") {
        cmp = a.title.localeCompare(b.title);
      } else {
        cmp = a.createdAt.localeCompare(b.createdAt);
      }
      return order === "asc" ? cmp : -cmp;
    });
  }

  async function loadWorks() {
    clearThumbnailCache();
    if (filterTags.length > 0) {
      const filtered: WorkSummary[] = await invoke("search_works_by_tags", {
        tagIds: filterTags.map((t) => t.id),
        mode: tagSearchMode,
      });
      works = sortWorksClientSide(filtered, sortField, sortOrder);
    } else {
      works = await invoke("list_works", {
        sortBy: sortField,
        sortOrder: sortOrder,
      });
    }
    onWorksLoaded?.(works.map((w) => w.id));
  }

  function addFilterTag(tag: Tag) {
    onFilterTagsChange([...filterTags, tag]);
  }

  function removeFilterTag(tagId: number) {
    onFilterTagsChange(filterTags.filter((t) => t.id !== tagId));
  }

  function toggleSearchMode() {
    onTagSearchModeChange(tagSearchMode === "and" ? "or" : "and");
  }

  $effect(() => {
    void reloadTrigger;
    void sortField;
    void sortOrder;
    void filterTags;
    void tagSearchMode;
    loadWorks();
  });

  function handleContextMenu(workId: number, e: MouseEvent) {
    contextMenu = { x: e.clientX, y: e.clientY, workId };
  }

  async function openEditDialog(workId: number) {
    contextMenu = null;
    try {
      const detail: WorkDetail = await invoke("get_work", { workId });
      editingWork = detail;
    } catch (e) {
      console.error("Failed to get work:", e);
    }
  }

  function openAddToPlaylistDialog(workId: number) {
    contextMenu = null;
    addingToPlaylistWorkId = workId;
  }

  async function openDeleteDialog(workId: number) {
    contextMenu = null;
    const work = works.find((w) => w.id === workId);
    if (!work) return;
    try {
      const settings = await invoke<AppSettings>("get_settings");
      deleteFileAction = settings.deleteFileAction;
    } catch {
      deleteFileAction = "ask";
    }
    // trash はライブラリルート配下の .trash へ退避するため、
    // ライブラリにフォルダが未設定だと実行時に必ず失敗する。事前に ask 扱いへ倒す。
    if (deleteFileAction === "trash" && !libraryPath) {
      deleteFileAction = "ask";
    }
    selectedFileAction = "none";
    let detail: WorkDetail;
    try {
      detail = await invoke("get_work", { workId });
    } catch (e) {
      console.error("Failed to get work:", e);
      addToast(
        "error",
        `削除対象のパス取得に失敗したため削除を中止しました: ${e}`,
      );
      return;
    }
    deletingWork = {
      id: work.id,
      title: work.title,
      path: detail.path,
      workType: detail.workType,
    };
  }

  function resolveEffectiveDeleteAction(): DeleteFileAction | "none" {
    if (deleteFileAction === "ask") return selectedFileAction;
    return deleteFileAction;
  }

  async function handleDelete() {
    if (!deletingWork) return;
    const { id, title } = deletingWork;
    try {
      await invoke("delete_work", {
        workId: id,
        fileAction: resolveEffectiveDeleteAction(),
      });
      addToast("success", `「${title}」を削除しました`);
      deletingWork = null;
      loadWorks();
    } catch (e) {
      addToast("error", `削除に失敗しました: ${e}`);
    }
  }

  function handleSort(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    switch (value) {
      case "created_at_desc":
        sortField = "created_at";
        sortOrder = "desc";
        break;
      case "created_at_asc":
        sortField = "created_at";
        sortOrder = "asc";
        break;
      case "title_asc":
        sortField = "title";
        sortOrder = "asc";
        break;
      case "title_desc":
        sortField = "title";
        sortOrder = "desc";
        break;
    }
  }
</script>

<div class="toolbar">
  <div class="sort-control">
    <label for="sort-select">Sort:</label>
    <select
      id="sort-select"
      value="{sortField}_{sortOrder}"
      onchange={handleSort}
    >
      <option value="created_at_desc">Date (Newest)</option>
      <option value="created_at_asc">Date (Oldest)</option>
      <option value="title_asc">Title (A-Z)</option>
      <option value="title_desc">Title (Z-A)</option>
    </select>
  </div>
  <div class="filter-section">
    {#each filterTags as tag (tag.id)}
      <span class="tag-badge" data-category={tag.category}>
        {tag.name}
        <button
          class="tag-badge-remove"
          onclick={() => removeFilterTag(tag.id)}
        >
          &times;
        </button>
      </span>
    {/each}
    <TagInput
      variant="light"
      placeholder="タグで絞り込み..."
      onSelectTag={addFilterTag}
      excludeTagIds={filterTags.map((t) => t.id)}
    />
    {#if filterTags.length >= 2}
      <button class="filter-mode-select" onclick={toggleSearchMode}>
        {tagSearchMode === "and" ? "AND" : "OR"}
      </button>
    {/if}
  </div>
  <span class="work-count">{works.length} works</span>
</div>

<div class="grid-container" bind:clientWidth={containerWidth}>
  {#if rows.length > 0}
    <VList data={rows} getKey={(_, i) => i} itemSize={280}>
      {#snippet children(row)}
        <div
          class="grid-row"
          style="gap: {GAP}px; grid-template-columns: repeat({columnCount}, {CARD_WIDTH}px);"
        >
          {#each row as work (work.id)}
            <WorkCard
              {work}
              onclick={onSelectWork}
              oncontextmenu={handleContextMenu}
            />
          {/each}
        </div>
      {/snippet}
    </VList>
  {:else}
    <div class="empty-state">
      <p>
        作品がありません。「+
        取り込み」または「一括取り込み」で作品を追加してください。
      </p>
    </div>
  {/if}
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={[
      {
        label: "メタデータを編集",
        action: () => openEditDialog(contextMenu!.workId),
      },
      {
        label: "プレイリストに追加",
        action: () => openAddToPlaylistDialog(contextMenu!.workId),
      },
      {
        label: "削除",
        action: () => openDeleteDialog(contextMenu!.workId),
      },
    ]}
    onClose={() => (contextMenu = null)}
  />
{/if}

{#if addingToPlaylistWorkId !== null}
  <AddToPlaylistDialog
    workId={addingToPlaylistWorkId}
    onClose={() => (addingToPlaylistWorkId = null)}
    {onPlaylistCreated}
  />
{/if}

{#if deletingWork}
  <ConfirmDialog
    title="作品の削除"
    message="「{deletingWork.title}」を削除しますか？この操作は取り消せません。"
    confirmLabel="削除"
    danger={true}
    onConfirm={handleDelete}
    onCancel={() => (deletingWork = null)}
  >
    {#snippet extra()}
      {@const path = deletingWork!.path}
      {#if deleteFileAction === "delete"}
        <p class="confirm-dialog-file-action-note">
          ※ 以下のローカルファイルを完全に削除します（元に戻せません）:<br
          /><code class="confirm-dialog-path">{path}</code>
        </p>
        <p class="confirm-dialog-file-action-note">
          ※ 上記のファイルはライブラリ管理外の実データである可能性があります。
        </p>
        {#if deletingWork!.workType === "folder"}
          <p class="confirm-dialog-file-action-note">
            ※
            対象はフォルダのため、配下のライブラリ未登録のファイル・サブフォルダも含めてすべて削除されます。
          </p>
        {/if}
      {:else if deleteFileAction === "trash"}
        <p class="confirm-dialog-file-action-note">
          ※ 以下のローカルファイルを非追跡ディレクトリへ移動します:<br /><code
            class="confirm-dialog-path">{path}</code
          >
        </p>
      {:else if deleteFileAction === "ask"}
        <p class="confirm-dialog-path-note">
          対象パス: <code class="confirm-dialog-path">{path}</code>
        </p>
        <div class="confirm-dialog-file-action">
          <label class="confirm-dialog-radio">
            <input
              type="radio"
              name="file-action"
              value="none"
              checked={selectedFileAction === "none"}
              onchange={() => (selectedFileAction = "none")}
            />
            メタデータのみ削除（ファイルは保持）
          </label>
          <label
            class="confirm-dialog-radio"
            class:confirm-dialog-radio-disabled={!libraryPath}
          >
            <input
              type="radio"
              name="file-action"
              value="trash"
              checked={selectedFileAction === "trash"}
              disabled={!libraryPath}
              onchange={() => (selectedFileAction = "trash")}
            />
            ゴミ箱に退避する
            {#if !libraryPath}
              （ライブラリにフォルダが未設定のため利用できません）
            {/if}
          </label>
          <label class="confirm-dialog-radio">
            <input
              type="radio"
              name="file-action"
              value="delete"
              checked={selectedFileAction === "delete"}
              onchange={() => (selectedFileAction = "delete")}
            />
            ローカルファイルを完全削除する
          </label>
        </div>
        {#if selectedFileAction === "delete"}
          <p class="confirm-dialog-file-action-note">
            ※ 上記のファイルはライブラリ管理外の実データである可能性があります。
          </p>
          {#if deletingWork!.workType === "folder"}
            <p class="confirm-dialog-file-action-note">
              ※
              対象はフォルダのため、配下のライブラリ未登録のファイル・サブフォルダも含めてすべて削除されます。
            </p>
          {/if}
        {/if}
      {/if}
    {/snippet}
  </ConfirmDialog>
{/if}

{#if editingWork}
  <EditWorkDialog
    work={editingWork}
    onClose={() => (editingWork = null)}
    onUpdated={() => loadWorks()}
  />
{/if}
