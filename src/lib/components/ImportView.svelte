<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type {
    ImportMode,
    ImportRequest,
    ImportResult,
    ParsedMetadata,
  } from "../types";

  interface Props {
    onBack: () => void;
    onImported: () => void;
  }

  let { onBack, onImported }: Props = $props();

  type Step = "select" | "metadata" | "importing" | "done" | "error";

  let step = $state<Step>("select");
  let sourcePath = $state("");
  let title = $state("");
  let artist = $state("");
  let year = $state("");
  let genre = $state("");
  let circle = $state("");
  let origin = $state("");
  let mode = $state<ImportMode>("copy");
  let previewPath = $state<string | null>(null);
  let result = $state<ImportResult | null>(null);
  let errorMessage = $state("");
  let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let previewRequestId = 0;

  async function selectFolder() {
    const selected = await open({ directory: true });
    if (!selected) return;

    sourcePath = selected;
    const sep = selected.includes("\\") ? "\\" : "/";
    const folderName = selected.split(sep).pop() ?? selected;

    try {
      const parsed = await invoke<ParsedMetadata>("parse_folder_name", {
        folderName,
      });
      title = parsed.title;
      artist = parsed.artist ?? "";
    } catch {
      title = folderName;
      artist = "";
    }

    step = "metadata";
    updatePreview();
  }

  function buildMetadata() {
    return {
      title: title.trim(),
      artist: artist.trim() || null,
      year: year.trim() ? parseInt(year.trim(), 10) : null,
      genre: genre.trim() || null,
      circle: circle.trim() || null,
      origin: origin.trim() || null,
    };
  }

  async function updatePreview() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      if (!title.trim()) {
        previewPath = null;
        return;
      }
      const requestId = ++previewRequestId;
      try {
        const path = await invoke<string>("preview_import_path", {
          metadata: buildMetadata(),
        });
        if (requestId !== previewRequestId) return;
        previewPath = path;
      } catch {
        if (requestId !== previewRequestId) return;
        previewPath = null;
      }
    }, 300);
  }

  async function executeImport() {
    step = "importing";
    try {
      const request: ImportRequest = {
        sourcePath,
        ...buildMetadata(),
        mode,
      };
      result = await invoke<ImportResult>("import_work", { request });
      step = "done";
    } catch (e) {
      errorMessage = String(e);
      step = "error";
    }
  }

  function resetForm() {
    step = "select";
    sourcePath = "";
    title = "";
    artist = "";
    year = "";
    genre = "";
    circle = "";
    origin = "";
    mode = "copy";
    previewPath = null;
    result = null;
    errorMessage = "";
  }
</script>

<main class="container">
  <div class="app-header">
    <button class="settings-back-btn" onclick={onBack}>← ライブラリ</button>
    <h1>作品取り込み</h1>
  </div>

  {#if step === "select"}
    <div class="import-content">
      <section class="import-section">
        <h2>フォルダを選択</h2>
        <p class="import-description">
          取り込む画像フォルダを選択してください。
        </p>
        <button class="import-select-btn" onclick={selectFolder}>
          フォルダを選択...
        </button>
      </section>
    </div>
  {:else if step === "metadata"}
    <div class="import-content">
      <section class="import-section">
        <h2>メタデータ入力</h2>
        <p class="import-description">取り込む作品の情報を入力してください。</p>

        <div class="import-source-path">
          <span class="import-label">取り込み元:</span>
          <code>{sourcePath}</code>
        </div>

        <div class="import-form">
          <div class="import-field">
            <label class="import-label" for="import-title"
              >タイトル <span class="import-required">*</span></label
            >
            <input
              id="import-title"
              type="text"
              class="settings-input"
              bind:value={title}
              oninput={updatePreview}
              placeholder="作品タイトル"
            />
          </div>

          <div class="import-field">
            <label class="import-label" for="import-artist">アーティスト</label>
            <input
              id="import-artist"
              type="text"
              class="settings-input"
              bind:value={artist}
              oninput={updatePreview}
              placeholder="アーティスト名"
            />
          </div>

          <div class="import-field-row">
            <div class="import-field">
              <label class="import-label" for="import-year">年</label>
              <input
                id="import-year"
                type="text"
                class="settings-input"
                bind:value={year}
                oninput={updatePreview}
                placeholder="2025"
              />
            </div>
            <div class="import-field">
              <label class="import-label" for="import-genre">ジャンル</label>
              <input
                id="import-genre"
                type="text"
                class="settings-input"
                bind:value={genre}
                oninput={updatePreview}
                placeholder="ジャンル"
              />
            </div>
          </div>

          <div class="import-field-row">
            <div class="import-field">
              <label class="import-label" for="import-circle">サークル</label>
              <input
                id="import-circle"
                type="text"
                class="settings-input"
                bind:value={circle}
                oninput={updatePreview}
                placeholder="サークル名"
              />
            </div>
            <div class="import-field">
              <label class="import-label" for="import-origin">出典</label>
              <input
                id="import-origin"
                type="text"
                class="settings-input"
                bind:value={origin}
                oninput={updatePreview}
                placeholder="出典"
              />
            </div>
          </div>

          <div class="import-field">
            <label class="import-label">取り込みモード</label>
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
        </div>

        {#if previewPath}
          <div class="template-preview">
            <span class="template-preview-label">配置先:</span>
            <code class="template-preview-path">{previewPath}</code>
          </div>
        {/if}

        <div class="import-actions">
          <button class="settings-back-btn" onclick={resetForm}>
            ← 戻る
          </button>
          <button
            class="import-execute-btn"
            onclick={executeImport}
            disabled={!title.trim()}
          >
            取り込み実行
          </button>
        </div>
      </section>
    </div>
  {:else if step === "importing"}
    <div class="import-content">
      <div class="import-loading">取り込み中...</div>
    </div>
  {:else if step === "done"}
    <div class="import-content">
      <section class="import-section">
        <h2>取り込み完了</h2>
        {#if result}
          <p class="import-success">
            {result.pageCount}ページの作品を取り込みました。
          </p>
          <div class="import-source-path">
            <span class="import-label">配置先:</span>
            <code>{result.destinationPath}</code>
          </div>
        {/if}
        <div class="import-actions">
          <button
            class="settings-back-btn"
            onclick={() => {
              onImported();
              onBack();
            }}
          >
            ← ライブラリへ戻る
          </button>
          <button class="import-select-btn" onclick={resetForm}>
            別の作品を取り込む
          </button>
        </div>
      </section>
    </div>
  {:else if step === "error"}
    <div class="import-content">
      <section class="import-section">
        <h2>エラー</h2>
        <p class="import-error">{errorMessage}</p>
        <div class="import-actions">
          <button class="import-select-btn" onclick={resetForm}>
            やり直す
          </button>
        </div>
      </section>
    </div>
  {/if}
</main>
