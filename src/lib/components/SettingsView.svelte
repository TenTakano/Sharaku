<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Channel } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type {
    AppSettings,
    TemplateValidation,
    RelocationPreview,
    RelocationProgress,
  } from "../types";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  let libraryRoot = $state("");
  let directoryTemplate = $state("");
  let typeLabelImage = $state("");
  let typeLabelFolder = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let message = $state<{ type: "success" | "error"; text: string } | null>(
    null,
  );
  let templateValidation = $state<TemplateValidation>({
    valid: true,
    error: null,
  });
  let templatePreview = $state<string | null>(null);
  let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let validationRequestId = 0;

  let relocationPreviews = $state<RelocationPreview[]>([]);
  let showRelocationDialog = $state(false);
  let relocating = $state(false);
  let relocationProgress = $state<RelocationProgress | null>(null);

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      libraryRoot = settings.libraryRoot ?? "";
      directoryTemplate = settings.directoryTemplate ?? "";
      typeLabelImage = settings.typeLabelImage;
      typeLabelFolder = settings.typeLabelFolder;
      if (directoryTemplate) {
        await validateAndPreviewTemplate(directoryTemplate);
      }
    } catch (e) {
      message = { type: "error", text: `設定の読み込みに失敗しました: ${e}` };
    } finally {
      loading = false;
    }
  }

  async function browseLibraryRoot() {
    const selected = await open({ directory: true });
    if (selected) {
      libraryRoot = selected;
    }
  }

  async function saveLibraryRoot() {
    if (!libraryRoot.trim()) return;
    saving = true;
    message = null;
    try {
      await invoke("set_library_root", { path: libraryRoot.trim() });
      message = { type: "success", text: "ライブラリルートを保存しました" };
    } catch (e) {
      message = { type: "error", text: `保存に失敗しました: ${e}` };
    } finally {
      saving = false;
    }
  }

  async function saveDirectoryTemplate() {
    saving = true;
    message = null;
    try {
      const previews = await invoke<RelocationPreview[]>("preview_relocation", {
        newTemplate: directoryTemplate.trim(),
      });
      if (previews.length === 0) {
        await invoke("set_directory_template", {
          template: directoryTemplate.trim(),
        });
        message = {
          type: "success",
          text: "ディレクトリテンプレートを保存しました",
        };
      } else {
        relocationPreviews = previews;
        showRelocationDialog = true;
      }
    } catch (e) {
      message = { type: "error", text: `保存に失敗しました: ${e}` };
    } finally {
      saving = false;
    }
  }

  function cancelRelocation() {
    showRelocationDialog = false;
    relocationPreviews = [];
    relocationProgress = null;
  }

  async function executeRelocation() {
    relocating = true;
    relocationProgress = null;
    try {
      const channel = new Channel<RelocationProgress>();
      channel.onmessage = (progress) => {
        relocationProgress = progress;
      };
      await invoke("relocate_works", {
        newTemplate: directoryTemplate.trim(),
        onProgress: channel,
      });
      showRelocationDialog = false;
      relocationPreviews = [];
      message = {
        type: "success",
        text: "テンプレートを保存し、作品を再配置しました",
      };
    } catch (e) {
      message = { type: "error", text: `再配置に失敗しました: ${e}` };
    } finally {
      relocating = false;
      relocationProgress = null;
    }
  }

  async function validateAndPreviewTemplate(value: string) {
    const trimmed = value.trim();
    if (!trimmed) {
      templateValidation = { valid: true, error: null };
      templatePreview = null;
      return;
    }
    const requestId = ++validationRequestId;
    try {
      await invoke("validate_template", { template: trimmed });
      if (requestId !== validationRequestId) return;
      templateValidation = { valid: true, error: null };
      try {
        const preview = await invoke<string>("preview_template", {
          template: trimmed,
        });
        if (requestId !== validationRequestId) return;
        templatePreview = preview;
      } catch {
        if (requestId !== validationRequestId) return;
        templatePreview = null;
      }
    } catch (e) {
      if (requestId !== validationRequestId) return;
      templateValidation = { valid: false, error: String(e) };
      templatePreview = null;
    }
  }

  async function saveTypeLabels() {
    saving = true;
    message = null;
    try {
      await invoke("set_type_labels", {
        imageLabel: typeLabelImage.trim(),
        folderLabel: typeLabelFolder.trim(),
      });
      message = { type: "success", text: "作品種別ラベルを保存しました" };
      if (directoryTemplate) {
        await validateAndPreviewTemplate(directoryTemplate);
      }
    } catch (e) {
      message = { type: "error", text: `保存に失敗しました: ${e}` };
    } finally {
      saving = false;
    }
  }

  function onTemplateInput() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      validateAndPreviewTemplate(directoryTemplate);
    }, 300);
  }

  $effect(() => {
    loadSettings();
  });
</script>

<main class="container">
  <div class="app-header">
    <button class="settings-back-btn" onclick={onBack}>← ライブラリ</button>
    <h1>設定</h1>
  </div>

  {#if loading}
    <p class="settings-loading">読み込み中...</p>
  {:else}
    <div class="settings-content">
      <section class="settings-section">
        <h2>ライブラリルート</h2>
        <p class="settings-description">
          作品ファイルを管理するルートディレクトリを指定します。
        </p>
        <div class="settings-field-row">
          <input
            type="text"
            class="settings-input"
            bind:value={libraryRoot}
            placeholder="/path/to/library"
            disabled={saving}
          />
          <button
            class="settings-browse-btn"
            onclick={browseLibraryRoot}
            disabled={saving}
          >
            参照...
          </button>
          <button
            class="settings-save-btn"
            onclick={saveLibraryRoot}
            disabled={saving || !libraryRoot.trim()}
          >
            保存
          </button>
        </div>
      </section>

      <section class="settings-section">
        <h2>ディレクトリテンプレート</h2>
        <p class="settings-description">
          作品取り込み時のフォルダ配置パターンを指定します。<br />
          使用可能なプレースホルダー:
          <code>{"{title}"}</code>, <code>{"{artist}"}</code>,
          <code>{"{year}"}</code>,
          <code>{"{genre}"}</code>, <code>{"{circle}"}</code>,
          <code>{"{origin}"}</code>, <code>{"{type}"}</code>
        </p>
        <div class="settings-field-row">
          <input
            type="text"
            class="settings-input"
            class:settings-input-error={!templateValidation.valid}
            bind:value={directoryTemplate}
            oninput={onTemplateInput}
            placeholder={"{artist}/{title}"}
            disabled={saving}
          />
          <button
            class="settings-save-btn"
            onclick={saveDirectoryTemplate}
            disabled={saving || !templateValidation.valid}
          >
            保存
          </button>
        </div>
        {#if !templateValidation.valid && templateValidation.error}
          <p class="template-error">{templateValidation.error}</p>
        {/if}
        {#if templateValidation.valid && templatePreview}
          <div class="template-preview">
            <span class="template-preview-label">プレビュー:</span>
            <code class="template-preview-path">{templatePreview}</code>
          </div>
        {/if}
      </section>

      <section class="settings-section">
        <h2>作品種別ラベル</h2>
        <p class="settings-description">
          テンプレートの <code>{"{type}"}</code>
          に使用するラベルをカスタマイズできます。
        </p>
        <div class="type-label-fields">
          <div class="type-label-row">
            <label class="type-label-name" for="type-label-image"
              >画像作品:</label
            >
            <input
              id="type-label-image"
              type="text"
              class="settings-input type-label-input"
              bind:value={typeLabelImage}
              placeholder="Image"
              disabled={saving}
            />
          </div>
          <div class="type-label-row">
            <label class="type-label-name" for="type-label-folder"
              >フォルダ作品:</label
            >
            <input
              id="type-label-folder"
              type="text"
              class="settings-input type-label-input"
              bind:value={typeLabelFolder}
              placeholder="Folder"
              disabled={saving}
            />
          </div>
          <button
            class="settings-save-btn"
            onclick={saveTypeLabels}
            disabled={saving ||
              !typeLabelImage.trim() ||
              !typeLabelFolder.trim()}
          >
            保存
          </button>
        </div>
      </section>
    </div>

    {#if message}
      <p class="settings-message {message.type}">{message.text}</p>
    {/if}
  {/if}

  {#if showRelocationDialog}
    <div class="relocation-overlay">
      <div class="relocation-dialog">
        {#if !relocating}
          <h2>作品の再配置</h2>
          <p class="relocation-warning">
            テンプレートの変更により、{relocationPreviews.length}
            件の作品ディレクトリが移動されます。
          </p>
          <div class="relocation-preview-list">
            {#each relocationPreviews as item (item.workId)}
              <div class="relocation-preview-item">
                <span class="relocation-preview-title">{item.title}</span>
                <div class="relocation-paths">
                  <code class="relocation-path-old">{item.oldPath}</code>
                  <span class="relocation-arrow">→</span>
                  <code class="relocation-path-new">{item.newPath}</code>
                </div>
              </div>
            {/each}
          </div>
          <div class="relocation-actions">
            <button class="relocation-cancel-btn" onclick={cancelRelocation}>
              キャンセル
            </button>
            <button class="relocation-execute-btn" onclick={executeRelocation}>
              実行
            </button>
          </div>
        {:else}
          <h2>再配置中...</h2>
          {#if relocationProgress}
            {#if relocationProgress.type === "started"}
              <p class="relocation-progress">
                {relocationProgress.total} 件の作品を処理します...
              </p>
            {:else if relocationProgress.type === "moving"}
              <p class="relocation-progress">
                ({relocationProgress.current}/{relocationProgress.total})
                {relocationProgress.title}
              </p>
              <progress
                value={relocationProgress.current}
                max={relocationProgress.total}
              ></progress>
            {:else if relocationProgress.type === "completed"}
              <p class="relocation-progress">
                完了: {relocationProgress.relocated} 件移動,
                {relocationProgress.skipped} 件スキップ,
                {relocationProgress.failed} 件失敗
              </p>
            {:else if relocationProgress.type === "error"}
              <p class="relocation-progress relocation-progress-error">
                {relocationProgress.message}
              </p>
            {/if}
          {:else}
            <p class="relocation-progress">準備中...</p>
          {/if}
        {/if}
      </div>
    </div>
  {/if}
</main>
