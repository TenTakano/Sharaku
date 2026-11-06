<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { addToast } from "../stores/toast.svelte";
  import { debounce } from "../utils/debounce";
  import { createLatestRequestGuard } from "../utils/latestRequest";
  import type {
    AppSettings,
    ResourceMode,
    DeleteFileAction,
    TemplateValidation,
    ViewKind,
  } from "../types";

  interface Props {
    libraryId: string;
    libraryName: string;
    libraryPath: string | null;
    onNavigate: (view: ViewKind) => void;
    onDeleteLibrary: () => void;
  }

  let {
    libraryId,
    libraryName,
    libraryPath,
    onNavigate,
    onDeleteLibrary,
  }: Props = $props();

  let resourceMode = $state<ResourceMode>("full");
  let deleteFileAction = $state<DeleteFileAction>("ask");
  let directoryTemplate = $state("");
  let typeLabelImage = $state("");
  let typeLabelFolder = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let templateValidation = $state<TemplateValidation>({
    valid: true,
    error: null,
  });
  let templatePreview = $state<string | null>(null);
  const validationRequestGuard = createLatestRequestGuard();

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      resourceMode = settings.resourceMode;
      deleteFileAction = settings.deleteFileAction;
      directoryTemplate = settings.directoryTemplate ?? "";
      typeLabelImage = settings.typeLabelImage;
      typeLabelFolder = settings.typeLabelFolder;
      if (directoryTemplate) {
        await validateAndPreviewTemplate(directoryTemplate);
      }
    } catch (e) {
      addToast("error", `設定の読み込みに失敗しました: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function setResourceMode(mode: ResourceMode) {
    try {
      await invoke("set_resource_mode", { mode });
      resourceMode = mode;
    } catch (e) {
      addToast("error", `モードの変更に失敗しました: ${e}`);
    }
  }

  async function setDeleteFileAction(action: DeleteFileAction) {
    try {
      await invoke("set_delete_file_action", { action });
      deleteFileAction = action;
    } catch (e) {
      addToast("error", `設定の変更に失敗しました: ${e}`);
    }
  }

  async function saveDirectoryTemplate() {
    saving = true;
    try {
      await invoke("set_directory_template", {
        template: directoryTemplate.trim(),
      });
      addToast("success", "ディレクトリテンプレートを保存しました");
    } catch (e) {
      addToast("error", `保存に失敗しました: ${e}`);
    } finally {
      saving = false;
    }
  }

  async function validateAndPreviewTemplate(value: string) {
    const trimmed = value.trim();
    if (!trimmed) {
      templateValidation = { valid: true, error: null };
      templatePreview = null;
      return;
    }
    const requestId = validationRequestGuard.next();
    try {
      await invoke("validate_template", { template: trimmed });
      if (!validationRequestGuard.isLatest(requestId)) return;
      templateValidation = { valid: true, error: null };
      try {
        const preview = await invoke<string>("preview_template", {
          template: trimmed,
        });
        if (!validationRequestGuard.isLatest(requestId)) return;
        templatePreview = preview;
      } catch {
        if (!validationRequestGuard.isLatest(requestId)) return;
        templatePreview = null;
      }
    } catch (e) {
      if (!validationRequestGuard.isLatest(requestId)) return;
      templateValidation = { valid: false, error: String(e) };
      templatePreview = null;
    }
  }

  async function saveTypeLabels() {
    saving = true;
    try {
      await invoke("set_type_labels", {
        imageLabel: typeLabelImage.trim(),
        folderLabel: typeLabelFolder.trim(),
      });
      addToast("success", "作品種別ラベルを保存しました");
      if (directoryTemplate) {
        await validateAndPreviewTemplate(directoryTemplate);
      }
    } catch (e) {
      addToast("error", `保存に失敗しました: ${e}`);
    } finally {
      saving = false;
    }
  }

  const debouncedValidateAndPreview = debounce((value: string) => {
    validateAndPreviewTemplate(value);
  }, 300);

  function onTemplateInput() {
    debouncedValidateAndPreview(directoryTemplate);
  }

  let showDeleteConfirm = $state(false);
  let deleting = $state(false);

  async function deleteLibrary() {
    deleting = true;
    try {
      await invoke("remove_library", { id: libraryId });
      showDeleteConfirm = false;
      onDeleteLibrary();
    } catch (e) {
      addToast("error", `ライブラリの削除に失敗しました: ${e}`);
    } finally {
      deleting = false;
    }
  }

  $effect(() => {
    loadSettings();
  });
</script>

{#if loading}
  <p class="settings-loading">読み込み中...</p>
{:else}
  <div class="settings-scroll">
    <div class="settings-content">
      <section class="settings-section">
        <h2>ライブラリルート</h2>
        <p class="settings-description">
          現在のライブラリの保存先ディレクトリです。
        </p>
        <div class="settings-field-row">
          <code class="settings-library-path">{libraryPath ?? "未設定"}</code>
        </div>
      </section>

      <section class="settings-section">
        <h2>リソース管理モード</h2>
        <p class="settings-description">作品ファイルの管理方法を選択します。</p>
        <div class="radio-option-group">
          <label class="radio-option">
            <input
              type="radio"
              name="resource-mode"
              checked={resourceMode === "metadata_only"}
              onchange={() => setResourceMode("metadata_only")}
            />
            <span class="radio-option-label">メタデータのみ管理</span>
            <span class="radio-option-desc"
              >ファイルを移動せず、メタデータのみ管理</span
            >
          </label>
          <label class="radio-option">
            <input
              type="radio"
              name="resource-mode"
              checked={resourceMode === "full"}
              onchange={() => setResourceMode("full")}
            />
            <span class="radio-option-label">すべて管理</span>
            <span class="radio-option-desc"
              >テンプレートに基づきファイルを配置</span
            >
          </label>
        </div>

        <div
          class="resource-mode-sub-settings"
          class:settings-section-disabled={resourceMode === "metadata_only"}
        >
          <div class="settings-subsection">
            <h3>ディレクトリテンプレート</h3>
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
                disabled={saving || resourceMode === "metadata_only"}
              />
              <button
                class="settings-save-btn"
                onclick={saveDirectoryTemplate}
                disabled={saving ||
                  !templateValidation.valid ||
                  resourceMode === "metadata_only"}
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
          </div>

          <div class="settings-subsection">
            <h3>削除時のファイル処理</h3>
            <p class="settings-description">
              作品削除時にローカルファイルをどのように扱うかを設定します。
            </p>
            <div class="radio-option-group">
              <label class="radio-option">
                <input
                  type="radio"
                  name="delete-file-action"
                  checked={deleteFileAction === "ask"}
                  onchange={() => setDeleteFileAction("ask")}
                  disabled={resourceMode === "metadata_only"}
                />
                <span class="radio-option-label">実行時に確認する</span>
                <span class="radio-option-desc">削除時に処理方法を選択</span>
              </label>
              <label class="radio-option">
                <input
                  type="radio"
                  name="delete-file-action"
                  checked={deleteFileAction === "trash"}
                  onchange={() => setDeleteFileAction("trash")}
                  disabled={resourceMode === "metadata_only"}
                />
                <span class="radio-option-label"
                  >非追跡ディレクトリに退避させる</span
                >
                <span class="radio-option-desc"
                  >ライブラリ内の .trash ディレクトリに移動</span
                >
              </label>
              <label class="radio-option">
                <input
                  type="radio"
                  name="delete-file-action"
                  checked={deleteFileAction === "delete"}
                  onchange={() => setDeleteFileAction("delete")}
                  disabled={resourceMode === "metadata_only"}
                />
                <span class="radio-option-label">合わせて削除する</span>
                <span class="radio-option-desc"
                  >ローカルファイルを完全に削除</span
                >
              </label>
            </div>
          </div>

          <div class="settings-subsection">
            <h3>作品種別ラベル</h3>
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
                  disabled={saving || resourceMode === "metadata_only"}
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
                  disabled={saving || resourceMode === "metadata_only"}
                />
              </div>
              <button
                class="settings-save-btn"
                onclick={saveTypeLabels}
                disabled={saving ||
                  !typeLabelImage.trim() ||
                  !typeLabelFolder.trim() ||
                  resourceMode === "metadata_only"}
              >
                保存
              </button>
            </div>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h2>一括取り込み</h2>
        <p class="settings-description">
          フォルダを探索して、複数の作品を一括で取り込みます。
        </p>
        <button
          class="settings-bulk-import-btn"
          onclick={() => onNavigate("bulk-import")}
        >
          一括取り込みを開始
        </button>
      </section>

      <section class="settings-section settings-section-danger">
        <h2>ライブラリを削除</h2>
        <p class="settings-description">
          ライブラリの管理情報（作品・タグ等）を削除します。ファイルシステム上のファイルは削除されません。
        </p>
        <button
          class="settings-delete-library-btn"
          onclick={() => (showDeleteConfirm = true)}
        >
          ライブラリを削除
        </button>
      </section>
    </div>
  </div>
{/if}

{#if showDeleteConfirm}
  <div class="delete-library-overlay">
    <div class="delete-library-dialog">
      <p>
        ライブラリ「<strong>{libraryName}</strong>」を削除しますか？<br />
        管理情報（作品・タグ等）が削除されます。この操作は元に戻せません。
      </p>
      <div class="delete-library-actions">
        <button
          class="delete-library-cancel"
          onclick={() => (showDeleteConfirm = false)}
          disabled={deleting}
        >
          キャンセル
        </button>
        <button
          class="delete-library-confirm"
          onclick={deleteLibrary}
          disabled={deleting}
        >
          {#if deleting}
            削除中...
          {:else}
            削除する
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
