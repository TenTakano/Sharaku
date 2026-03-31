<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { SvelteSet, SvelteMap } from "svelte/reactivity";
  import { addToast } from "../stores/toast.svelte";
  import type {
    AppSettings,
    ResourceMode,
    DiscoveredFolder,
    DiscoverProgress,
    ImportMode,
    ImportRequest,
    EnqueueResult,
    UnregisteredEntry,
    ParsedMetadata,
  } from "../types";

  interface Props {
    initialEntries?: UnregisteredEntry[];
    initialRootPath?: string;
    initialDroppedPaths?: string[];
    onBack: () => void;
  }

  let { initialEntries, initialRootPath, initialDroppedPaths, onBack }: Props =
    $props();

  type Step = "discover" | "review";

  let resourceMode = $state<ResourceMode>("full");
  let step = $state<Step>("discover");
  let discovering = $state(false);
  let discoverStatus = $state("");
  let folders = $state<DiscoveredFolder[]>([]);
  let selected = new SvelteSet<number>();
  let editedTitles = new SvelteMap<number, string>();
  let editedArtists = new SvelteMap<number, string>();
  let mode = $state<ImportMode>("copy");
  let submitting = $state(false);

  async function discoverFromPath(rootPath: string) {
    discovering = true;
    discoverStatus = "探索中...";

    const channel = new Channel<DiscoverProgress>();
    channel.onmessage = (p) => {
      if (p.type === "scanning") {
        discoverStatus = `${p.scannedDirs} フォルダを探索中...`;
      } else if (p.type === "completed") {
        discoverStatus = `${p.found} 件のフォルダを検出`;
      }
    };

    try {
      const result = await invoke<DiscoveredFolder[]>("discover_folders", {
        rootPath,
        onProgress: channel,
      });
      folders = result;
      selected.clear();
      folders.forEach((f, i) => {
        if (!f.alreadyRegistered) {
          selected.add(i);
        }
      });
      editedTitles.clear();
      editedArtists.clear();
      step = "review";
    } catch (e) {
      discoverStatus = `エラー: ${e}`;
    } finally {
      discovering = false;
    }
  }

  async function discoverFromDroppedPaths(paths: string[]) {
    discovering = true;
    discoverStatus = "探索中...";

    const channel = new Channel<DiscoverProgress>();
    channel.onmessage = (p) => {
      if (p.type === "scanning") {
        discoverStatus = `${p.scannedDirs} フォルダを探索中...`;
      } else if (p.type === "completed") {
        discoverStatus = `${p.found} 件のフォルダを検出`;
      }
    };

    try {
      const result = await invoke<DiscoveredFolder[]>(
        "discover_dropped_paths",
        {
          paths,
          onProgress: channel,
        },
      );
      folders = result;
      selected.clear();
      folders.forEach((f, i) => {
        if (!f.alreadyRegistered) {
          selected.add(i);
        }
      });
      editedTitles.clear();
      editedArtists.clear();
      step = "review";
    } catch (e) {
      discoverStatus = `エラー: ${e}`;
    } finally {
      discovering = false;
    }
  }

  async function selectRootAndDiscover() {
    const rootPath = await open({ directory: true });
    if (!rootPath) return;
    await discoverFromPath(rootPath);
  }

  function getTitle(index: number): string {
    return editedTitles.get(index) ?? folders[index].parsedMetadata.title;
  }

  function getArtist(index: number): string {
    return (
      editedArtists.get(index) ?? folders[index].parsedMetadata.artist ?? ""
    );
  }

  function toggleSelect(index: number) {
    if (selected.has(index)) {
      selected.delete(index);
    } else {
      selected.add(index);
    }
  }

  function toggleAll() {
    if (selected.size === selectableCount) {
      selected.clear();
    } else {
      selected.clear();
      folders.forEach((f, i) => {
        if (!f.alreadyRegistered) selected.add(i);
      });
    }
  }

  let selectableCount = $derived(
    folders.filter((f) => !f.alreadyRegistered).length,
  );

  async function executeImport() {
    const requests: ImportRequest[] = [];
    for (const index of selected) {
      const folder = folders[index];
      requests.push({
        sourcePath: folder.path,
        title: getTitle(index),
        artist: getArtist(index) || null,
        year: null,
        genre: null,
        circle: null,
        origin: null,
        mode,
      });
    }

    submitting = true;
    try {
      await invoke<EnqueueResult>("enqueue_import", { requests });
      onBack();
    } catch (e) {
      addToast("error", String(e));
    } finally {
      submitting = false;
    }
  }

  function resetToDiscover() {
    step = "discover";
    folders = [];
    selected.clear();
    editedTitles.clear();
    editedArtists.clear();
    discoverStatus = "";
  }

  async function loadFromEntries(entries: UnregisteredEntry[]) {
    const parsed = await Promise.all(
      entries.map((entry) =>
        invoke<ParsedMetadata>("parse_folder_name", {
          folderName: entry.folderName,
        }),
      ),
    );
    folders = entries.map((entry, i) => ({
      path: entry.path,
      folderName: entry.folderName,
      imageCount: entry.imageCount,
      parsedMetadata: parsed[i],
      alreadyRegistered: false,
    }));
    selected.clear();
    folders.forEach((_, i) => selected.add(i));
    editedTitles.clear();
    editedArtists.clear();
    mode = "move";
    step = "review";
  }

  $effect(() => {
    invoke<AppSettings>("get_settings").then((settings) => {
      resourceMode = settings.resourceMode;
    });
    if (initialEntries) {
      loadFromEntries(initialEntries);
    } else if (initialDroppedPaths) {
      discoverFromDroppedPaths(initialDroppedPaths);
    } else if (initialRootPath) {
      discoverFromPath(initialRootPath);
    }
  });
</script>

{#if step === "discover"}
  <div class="import-content">
    <section class="import-section">
      <h2>探索するフォルダを選択</h2>
      <p class="import-description">
        ルートフォルダを選択すると、画像を含むサブフォルダを自動検出します。
      </p>
      <button
        class="import-select-btn"
        onclick={selectRootAndDiscover}
        disabled={discovering}
      >
        {discovering ? "探索中..." : "フォルダを選択..."}
      </button>
      {#if discoverStatus}
        <p class="bulk-discover-status">{discoverStatus}</p>
      {/if}
    </section>
  </div>
{:else if step === "review"}
  <div class="bulk-review-content">
    <section class="import-section">
      <h2>取り込み対象の確認</h2>
      <p class="import-description">
        {folders.length} フォルダ検出 / {selected.size} 件選択中
      </p>

      <div class="bulk-toolbar">
        <label class="bulk-select-all">
          <input
            type="checkbox"
            checked={selected.size === selectableCount && selectableCount > 0}
            onchange={toggleAll}
          />
          すべて選択
        </label>
        {#if resourceMode === "full"}
          <div class="import-mode-select">
            <label class="import-mode-option">
              <input type="radio" bind:group={mode} value="copy" />
              コピー
            </label>
            <label class="import-mode-option">
              <input type="radio" bind:group={mode} value="move" />
              移動
            </label>
          </div>
        {/if}
      </div>

      <div class="bulk-table-wrapper">
        <table class="bulk-table">
          <thead>
            <tr>
              <th class="bulk-th-check"></th>
              <th class="bulk-th-folder">フォルダ</th>
              <th class="bulk-th-count">画像数</th>
              <th class="bulk-th-title">タイトル</th>
              <th class="bulk-th-artist">アーティスト</th>
              <th class="bulk-th-status">状態</th>
            </tr>
          </thead>
          <tbody>
            {#each folders as folder, i (folder.path)}
              <tr class:bulk-row-disabled={folder.alreadyRegistered}>
                <td>
                  <input
                    type="checkbox"
                    checked={selected.has(i)}
                    disabled={folder.alreadyRegistered}
                    onchange={() => toggleSelect(i)}
                  />
                </td>
                <td class="bulk-cell-folder" title={folder.path}>
                  {folder.folderName}
                </td>
                <td class="bulk-cell-count">{folder.imageCount}</td>
                <td>
                  <input
                    type="text"
                    class="bulk-inline-input"
                    value={getTitle(i)}
                    disabled={folder.alreadyRegistered}
                    oninput={(e) =>
                      editedTitles.set(i, (e.target as HTMLInputElement).value)}
                  />
                </td>
                <td>
                  <input
                    type="text"
                    class="bulk-inline-input"
                    value={getArtist(i)}
                    disabled={folder.alreadyRegistered}
                    oninput={(e) =>
                      editedArtists.set(
                        i,
                        (e.target as HTMLInputElement).value,
                      )}
                  />
                </td>
                <td class="bulk-cell-status">
                  {#if folder.alreadyRegistered}
                    <span class="bulk-registered">登録済み</span>
                  {:else}
                    <span class="bulk-new">新規</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <div class="import-actions">
        <button
          class="settings-back-btn"
          onclick={initialEntries ? onBack : resetToDiscover}
        >
          ← 戻る
        </button>
        <button
          class="import-execute-btn"
          onclick={executeImport}
          disabled={selected.size === 0 || submitting}
        >
          {selected.size} 件を取り込み
        </button>
      </div>
    </section>
  </div>
{/if}
