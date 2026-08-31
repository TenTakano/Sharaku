<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { SvelteSet, SvelteMap } from "svelte/reactivity";
  import { addToast } from "../stores/toast.svelte";
  import {
    BULK_IMPORT_METADATA_FIELDS,
    type BulkImportMetadataField,
  } from "../utils/templatePlaceholders";
  import type {
    DiscoverResult,
    DiscoverProgress,
    ImportKind,
    ImportRequest,
    EnqueueResult,
    ParsedMetadata,
    SkippedFolder,
  } from "../types";

  const METADATA_FIELD_LABELS: Record<BulkImportMetadataField, string> = {
    year: "年",
    genre: "ジャンル",
    circle: "サークル",
    origin: "出典",
  };

  type BulkEntry = {
    kind: ImportKind;
    path: string;
    displayName: string;
    imageCount: number;
    parsedMetadata: ParsedMetadata;
    alreadyRegistered: boolean;
  };

  interface Props {
    initialRootPath?: string;
    initialDroppedPaths?: string[];
    onBack: () => void;
  }

  let { initialRootPath, initialDroppedPaths, onBack }: Props = $props();

  type Step = "discover" | "review";

  let step = $state<Step>("discover");
  let discovering = $state(false);
  let discoverStatus = $state("");
  let entries = $state<BulkEntry[]>([]);
  let skippedFolders = $state<SkippedFolder[]>([]);
  let showSkippedDetails = $state(false);
  let selected = new SvelteSet<number>();
  let editedTitles = new SvelteMap<number, string>();
  let editedArtists = new SvelteMap<number, string>();
  const editedMetadata: Record<
    BulkImportMetadataField,
    SvelteMap<number, string>
  > = {
    year: new SvelteMap(),
    genre: new SvelteMap(),
    circle: new SvelteMap(),
    origin: new SvelteMap(),
  };
  let submitting = $state(false);

  const activeMetadataFields = BULK_IMPORT_METADATA_FIELDS;

  function clearEditedMetadata() {
    for (const field of BULK_IMPORT_METADATA_FIELDS) {
      editedMetadata[field].clear();
    }
  }

  function getMetadataValue(
    index: number,
    field: BulkImportMetadataField,
  ): string {
    return editedMetadata[field].get(index) ?? "";
  }

  function setMetadataValue(
    index: number,
    field: BulkImportMetadataField,
    value: string,
  ) {
    editedMetadata[field].set(index, value);
  }

  function parseYearInput(value: string): number | null {
    const trimmed = value.trim();
    if (!trimmed) return null;
    const parsed = parseInt(trimmed, 10);
    return Number.isNaN(parsed) ? null : parsed;
  }

  function buildEntriesFromResult(result: DiscoverResult): BulkEntry[] {
    const folderEntries: BulkEntry[] = result.folders.map((f) => ({
      kind: "folder",
      path: f.path,
      displayName: f.folderName,
      imageCount: f.imageCount,
      parsedMetadata: f.parsedMetadata,
      alreadyRegistered: f.alreadyRegistered,
    }));
    const imageEntries: BulkEntry[] = result.images.map((img) => ({
      kind: "image",
      path: img.path,
      displayName: img.fileName,
      imageCount: 1,
      parsedMetadata: img.parsedMetadata,
      alreadyRegistered: img.alreadyRegistered,
    }));
    return [...folderEntries, ...imageEntries];
  }

  function applyDiscoverResult(result: DiscoverResult) {
    entries = buildEntriesFromResult(result);
    skippedFolders = result.skippedFolders;
    showSkippedDetails = false;
    selected.clear();
    entries.forEach((e, i) => {
      if (!e.alreadyRegistered) selected.add(i);
    });
    editedTitles.clear();
    editedArtists.clear();
    clearEditedMetadata();
    step = "review";
  }

  async function discoverFromPath(rootPath: string) {
    discovering = true;
    discoverStatus = "探索中...";

    const channel = new Channel<DiscoverProgress>();
    channel.onmessage = (p) => {
      if (p.type === "scanning") {
        discoverStatus = `${p.scannedDirs} フォルダを探索中...`;
      } else if (p.type === "completed") {
        discoverStatus = `${p.found} 件を検出`;
      }
    };

    try {
      const result = await invoke<DiscoverResult>("discover_folders", {
        rootPath,
        onProgress: channel,
      });
      applyDiscoverResult(result);
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
        discoverStatus = `${p.found} 件を検出`;
      }
    };

    try {
      const result = await invoke<DiscoverResult>("discover_dropped_paths", {
        paths,
        onProgress: channel,
      });
      applyDiscoverResult(result);
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
    return editedTitles.get(index) ?? entries[index].parsedMetadata.title;
  }

  function getArtist(index: number): string {
    return (
      editedArtists.get(index) ?? entries[index].parsedMetadata.artist ?? ""
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
      entries.forEach((e, i) => {
        if (!e.alreadyRegistered) selected.add(i);
      });
    }
  }

  let selectableCount = $derived(
    entries.filter((e) => !e.alreadyRegistered).length,
  );

  async function executeImport() {
    const requests: ImportRequest[] = [];
    for (const index of selected) {
      const entry = entries[index];
      requests.push({
        sourcePath: entry.path,
        title: getTitle(index).trim(),
        artist: getArtist(index).trim() || null,
        year: parseYearInput(getMetadataValue(index, "year")),
        genre: getMetadataValue(index, "genre").trim() || null,
        circle: getMetadataValue(index, "circle").trim() || null,
        origin: getMetadataValue(index, "origin").trim() || null,
        kind: entry.kind,
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
    entries = [];
    skippedFolders = [];
    showSkippedDetails = false;
    selected.clear();
    editedTitles.clear();
    editedArtists.clear();
    clearEditedMetadata();
    discoverStatus = "";
  }

  $effect(() => {
    if (initialDroppedPaths) {
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
        {entries.length} 件検出 / {selected.size} 件選択中
      </p>

      {#if skippedFolders.length > 0}
        {@const skippedImageCount = skippedFolders.reduce(
          (sum, f) => sum + f.imageCount,
          0,
        )}
        <div class="bulk-skipped-warning">
          <p class="bulk-skipped-message">
            {skippedFolders.length} 件の中間フォルダにある {skippedImageCount} 枚の画像はスキップされました。最下層フォルダのみが作品として取り込まれます。
          </p>
          <button
            class="bulk-skipped-toggle"
            onclick={() => (showSkippedDetails = !showSkippedDetails)}
          >
            {showSkippedDetails ? "詳細を隠す" : "詳細を表示"}
          </button>
          {#if showSkippedDetails}
            <ul class="bulk-skipped-list">
              {#each skippedFolders as sf (sf.path)}
                <li>
                  {sf.folderName}（{sf.imageCount} 枚）
                  <span class="bulk-skipped-path">{sf.path}</span>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}

      <div class="bulk-toolbar">
        <label class="bulk-select-all">
          <input
            type="checkbox"
            checked={selected.size === selectableCount && selectableCount > 0}
            onchange={toggleAll}
          />
          すべて選択
        </label>
      </div>

      <div class="bulk-table-wrapper">
        <table class="bulk-table">
          <thead>
            <tr>
              <th class="bulk-th-check"></th>
              <th class="bulk-th-kind">種別</th>
              <th class="bulk-th-folder">名前</th>
              <th class="bulk-th-count">画像数</th>
              <th class="bulk-th-title">タイトル</th>
              <th class="bulk-th-artist">アーティスト</th>
              {#each activeMetadataFields as field (field)}
                <th class="bulk-th-metadata">{METADATA_FIELD_LABELS[field]}</th>
              {/each}
              <th class="bulk-th-status">状態</th>
            </tr>
          </thead>
          <tbody>
            {#each entries as entry, i (entry.path)}
              <tr class:bulk-row-disabled={entry.alreadyRegistered}>
                <td>
                  <input
                    type="checkbox"
                    checked={selected.has(i)}
                    disabled={entry.alreadyRegistered}
                    onchange={() => toggleSelect(i)}
                  />
                </td>
                <td class="bulk-cell-kind">
                  {entry.kind === "folder" ? "フォルダ" : "画像"}
                </td>
                <td class="bulk-cell-folder" title={entry.path}>
                  {entry.displayName}
                </td>
                <td class="bulk-cell-count">{entry.imageCount}</td>
                <td>
                  <input
                    type="text"
                    class="bulk-inline-input"
                    aria-label="タイトル"
                    value={getTitle(i)}
                    disabled={entry.alreadyRegistered}
                    oninput={(e) =>
                      editedTitles.set(i, (e.target as HTMLInputElement).value)}
                  />
                </td>
                <td>
                  <input
                    type="text"
                    class="bulk-inline-input"
                    aria-label="アーティスト"
                    value={getArtist(i)}
                    disabled={entry.alreadyRegistered}
                    oninput={(e) =>
                      editedArtists.set(
                        i,
                        (e.target as HTMLInputElement).value,
                      )}
                  />
                </td>
                {#each activeMetadataFields as field (field)}
                  <td>
                    <input
                      type="text"
                      class="bulk-inline-input"
                      aria-label={METADATA_FIELD_LABELS[field]}
                      value={getMetadataValue(i, field)}
                      disabled={entry.alreadyRegistered}
                      oninput={(e) =>
                        setMetadataValue(
                          i,
                          field,
                          (e.target as HTMLInputElement).value,
                        )}
                    />
                  </td>
                {/each}
                <td class="bulk-cell-status">
                  {#if entry.alreadyRegistered}
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
          disabled={selected.size === 0 || submitting}
        >
          {selected.size} 件を取り込み
        </button>
      </div>
    </section>
  </div>
{/if}
