<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type {
    DiscoveredFolder,
    DiscoverProgress,
    ImportMode,
    ImportRequest,
    BulkImportProgress,
    BulkImportSummary,
  } from "../types";

  interface Props {
    onBack: () => void;
    onImported: () => void;
  }

  let { onBack, onImported }: Props = $props();

  type Step = "discover" | "review" | "importing" | "done";

  let step = $state<Step>("discover");
  let discovering = $state(false);
  let discoverStatus = $state("");
  let folders = $state<DiscoveredFolder[]>([]);
  let selected = $state<Set<number>>(new Set());
  let editedTitles = $state<Map<number, string>>(new Map());
  let editedArtists = $state<Map<number, string>>(new Map());
  let mode = $state<ImportMode>("copy");
  let importProgress = $state<BulkImportProgress | null>(null);
  let summary = $state<BulkImportSummary | null>(null);

  async function selectRootAndDiscover() {
    const rootPath = await open({ directory: true });
    if (!rootPath) return;

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
      const initialSelected = new Set<number>();
      folders.forEach((f, i) => {
        if (!f.alreadyRegistered) {
          initialSelected.add(i);
        }
      });
      selected = initialSelected;
      editedTitles = new Map();
      editedArtists = new Map();
      step = "review";
    } catch (e) {
      discoverStatus = `エラー: ${e}`;
    } finally {
      discovering = false;
    }
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
    const next = new Set(selected);
    if (next.has(index)) {
      next.delete(index);
    } else {
      next.add(index);
    }
    selected = next;
  }

  function toggleAll() {
    if (selected.size === selectableCount) {
      selected = new Set();
    } else {
      const next = new Set<number>();
      folders.forEach((f, i) => {
        if (!f.alreadyRegistered) next.add(i);
      });
      selected = next;
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

    step = "importing";
    importProgress = null;

    const channel = new Channel<BulkImportProgress>();
    channel.onmessage = (p) => {
      importProgress = p;
    };

    try {
      summary = await invoke<BulkImportSummary>("bulk_import", {
        requests,
        onProgress: channel,
      });
      step = "done";
    } catch (e) {
      summary = { succeeded: 0, failed: requests.length };
      step = "done";
    }
  }

  function handleDone() {
    onImported();
    onBack();
  }

  function resetToDiscover() {
    step = "discover";
    folders = [];
    selected = new Set();
    editedTitles = new Map();
    editedArtists = new Map();
    importProgress = null;
    summary = null;
    discoverStatus = "";
  }
</script>

<main class="container">
  <div class="app-header">
    <button class="settings-back-btn" onclick={onBack}>← ライブラリ</button>
    <h1>一括取り込み</h1>
  </div>

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
                        editedTitles.set(
                          i,
                          (e.target as HTMLInputElement).value,
                        )}
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
          <button class="settings-back-btn" onclick={resetToDiscover}>
            ← 戻る
          </button>
          <button
            class="import-execute-btn"
            onclick={executeImport}
            disabled={selected.size === 0}
          >
            {selected.size} 件を取り込み
          </button>
        </div>
      </section>
    </div>
  {:else if step === "importing"}
    <div class="import-content">
      <section class="import-section">
        <h2>取り込み中</h2>
        {#if importProgress && importProgress.type === "importing"}
          <p class="bulk-progress-text">
            {importProgress.current} / {importProgress.total}: {importProgress.title}
          </p>
          <progress max={importProgress.total} value={importProgress.current}
          ></progress>
        {:else if importProgress && importProgress.type === "started"}
          <p class="bulk-progress-text">
            {importProgress.total} 件の取り込みを開始...
          </p>
          <progress max={importProgress.total} value={0}></progress>
        {:else}
          <p class="import-loading">準備中...</p>
        {/if}
      </section>
    </div>
  {:else if step === "done"}
    <div class="import-content">
      <section class="import-section">
        <h2>取り込み完了</h2>
        {#if summary}
          <p class="import-success">
            成功: {summary.succeeded} 件 / 失敗: {summary.failed} 件
          </p>
        {/if}
        <div class="import-actions">
          <button class="settings-back-btn" onclick={handleDone}>
            ← ライブラリへ戻る
          </button>
          <button class="import-select-btn" onclick={resetToDiscover}>
            続けて取り込む
          </button>
        </div>
      </section>
    </div>
  {/if}
</main>
