<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Channel } from "@tauri-apps/api/core";
  import { addToast } from "../stores/toast.svelte";
  import type {
    AppSettings,
    ResourceMode,
    TemplateValidation,
    RelocationPreview,
    RelocationProgress,
    IntegrityReport,
    IntegrityCheckProgress,
    UnregisteredEntry,
  } from "../types";

  interface Props {
    libraryId: string;
    libraryName: string;
    libraryPath: string | null;
    onNavigate: (view: string) => void;
    onImportUnregistered: (entries: UnregisteredEntry[]) => void;
    onDeleteLibrary: () => void;
  }

  let {
    libraryId,
    libraryName,
    libraryPath,
    onNavigate,
    onImportUnregistered,
    onDeleteLibrary,
  }: Props = $props();

  let resourceMode = $state<ResourceMode>("full");
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
  let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let validationRequestId = 0;

  let savedDirectoryTemplate = $state("");
  let relocationPreviews = $state<RelocationPreview[]>([]);

  let integrityChecking = $state(false);
  let integrityProgress = $state<IntegrityCheckProgress | null>(null);
  let integrityReport = $state<IntegrityReport | null>(null);
  let showIntegrityDialog = $state(false);
  let deletingOrphans = $state(false);
  let showRelocationDialog = $state(false);
  let relocating = $state(false);
  let relocationProgress = $state<RelocationProgress | null>(null);

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      resourceMode = settings.resourceMode;
      directoryTemplate = settings.directoryTemplate ?? "";
      savedDirectoryTemplate = directoryTemplate;
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

  async function saveDirectoryTemplate() {
    saving = true;
    try {
      const previews = await invoke<RelocationPreview[]>("preview_relocation", {
        newTemplate: directoryTemplate.trim(),
      });
      if (previews.length === 0) {
        await invoke("set_directory_template", {
          template: directoryTemplate.trim(),
        });
        savedDirectoryTemplate = directoryTemplate;
        addToast("success", "ディレクトリテンプレートを保存しました");
      } else {
        relocationPreviews = previews;
        showRelocationDialog = true;
      }
    } catch (e) {
      addToast("error", `保存に失敗しました: ${e}`);
    } finally {
      saving = false;
    }
  }

  function cancelRelocation() {
    showRelocationDialog = false;
    relocationPreviews = [];
    relocationProgress = null;
    directoryTemplate = savedDirectoryTemplate;
    if (directoryTemplate) {
      validateAndPreviewTemplate(directoryTemplate);
    }
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
      savedDirectoryTemplate = directoryTemplate;
      addToast("success", "テンプレートを保存し、作品を再配置しました");
    } catch (e) {
      addToast("error", `再配置に失敗しました: ${e}`);
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

  function onTemplateInput() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      validateAndPreviewTemplate(directoryTemplate);
    }, 300);
  }

  async function runIntegrityCheck() {
    integrityChecking = true;
    integrityProgress = null;
    integrityReport = null;
    try {
      const channel = new Channel<IntegrityCheckProgress>();
      channel.onmessage = (progress) => {
        integrityProgress = progress;
      };
      const report = await invoke<IntegrityReport>("check_integrity", {
        onProgress: channel,
      });
      integrityReport = report;
      showIntegrityDialog = true;
    } catch (e) {
      addToast("error", `整合チェックに失敗しました: ${e}`);
    } finally {
      integrityChecking = false;
      integrityProgress = null;
    }
  }

  function closeIntegrityDialog() {
    showIntegrityDialog = false;
    integrityReport = null;
  }

  async function deleteOrphanWorks() {
    if (!integrityReport || integrityReport.orphanWorks.length === 0) return;
    deletingOrphans = true;
    try {
      const ids = integrityReport.orphanWorks.map((w) => w.id);
      const deleted = await invoke<number>("delete_orphan_works", { ids });
      addToast("success", `${deleted} 件の孤立レコードを削除しました`);
      showIntegrityDialog = false;
      integrityReport = null;
    } catch (e) {
      addToast("error", `削除に失敗しました: ${e}`);
    } finally {
      deletingOrphans = false;
    }
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
        <div class="resource-mode-select">
          <label class="resource-mode-option">
            <input
              type="radio"
              name="resource-mode"
              checked={resourceMode === "metadata_only"}
              onchange={() => setResourceMode("metadata_only")}
            />
            <span class="resource-mode-label">メタデータのみ管理</span>
            <span class="resource-mode-desc"
              >ファイルを移動せず、メタデータのみ管理</span
            >
          </label>
          <label class="resource-mode-option">
            <input
              type="radio"
              name="resource-mode"
              checked={resourceMode === "full"}
              onchange={() => setResourceMode("full")}
            />
            <span class="resource-mode-label">すべて管理</span>
            <span class="resource-mode-desc"
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

      <section class="settings-section">
        <h2>ライブラリ整合チェック</h2>
        <p class="settings-description">
          DBレコードとファイルシステムの不整合を検出します。
        </p>
        <p class="settings-description integrity-warning">
          ※
          ネットワークドライブが未接続の場合、正常なレコードが孤立として検出される可能性があります。
        </p>
        <button
          class="settings-bulk-import-btn"
          onclick={runIntegrityCheck}
          disabled={integrityChecking}
        >
          {#if integrityChecking}
            チェック中...
          {:else}
            整合チェックを実行
          {/if}
        </button>
        {#if integrityChecking && integrityProgress}
          <p class="integrity-progress">
            {#if integrityProgress.type === "checkingWorks"}
              作品レコードを確認中... ({integrityProgress.checked}/{integrityProgress.total})
            {:else if integrityProgress.type === "scanningDirectory"}
              ディレクトリを走査中... ({integrityProgress.scannedDirs} フォルダ)
            {/if}
          </p>
        {/if}
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

{#if showIntegrityDialog && integrityReport}
  <div class="integrity-overlay">
    <div class="integrity-dialog">
      <h2>整合チェック結果</h2>
      {#if integrityReport.orphanWorks.length === 0 && integrityReport.unregisteredEntries.length === 0}
        <p class="integrity-no-issues">問題は見つかりませんでした。</p>
        <p class="integrity-summary">
          全作品数: {integrityReport.totalWorks} 件
        </p>
      {:else}
        <p class="integrity-summary">
          全作品数: {integrityReport.totalWorks} 件 / 孤立レコード: {integrityReport
            .orphanWorks.length} 件 / 未登録フォルダ: {integrityReport
            .unregisteredEntries.length} 件
        </p>

        {#if integrityReport.orphanWorks.length > 0}
          <h3 class="integrity-section-title">
            孤立レコード（ファイルが存在しないDBレコード）
          </h3>
          <div class="integrity-list">
            {#each integrityReport.orphanWorks as work (work.id)}
              <div class="integrity-list-item">
                <span class="integrity-item-title">{work.title}</span>
                <code class="integrity-item-path">{work.path}</code>
              </div>
            {/each}
          </div>
        {/if}

        {#if integrityReport.unregisteredEntries.length > 0}
          <h3 class="integrity-section-title">
            未登録フォルダ（DBに登録されていないフォルダ）
          </h3>
          <div class="integrity-list">
            {#each integrityReport.unregisteredEntries as entry (entry.path)}
              <div class="integrity-list-item">
                <span class="integrity-item-title">{entry.folderName}</span>
                <span class="integrity-item-detail"
                  >画像: {entry.imageCount} 枚</span
                >
                <code class="integrity-item-path">{entry.path}</code>
              </div>
            {/each}
          </div>
        {/if}
      {/if}

      <div class="integrity-actions">
        <button
          class="integrity-close-btn"
          onclick={closeIntegrityDialog}
          disabled={deletingOrphans}
        >
          閉じる
        </button>
        {#if integrityReport.orphanWorks.length > 0}
          <button
            class="integrity-delete-btn"
            onclick={deleteOrphanWorks}
            disabled={deletingOrphans}
          >
            {#if deletingOrphans}
              削除中...
            {:else}
              孤立レコードを削除 ({integrityReport.orphanWorks.length}件)
            {/if}
          </button>
        {/if}
        {#if integrityReport.unregisteredEntries.length > 0}
          <button
            class="integrity-import-btn"
            onclick={() => {
              onImportUnregistered(integrityReport!.unregisteredEntries);
              closeIntegrityDialog();
            }}
            disabled={deletingOrphans}
          >
            未登録フォルダを取り込む ({integrityReport.unregisteredEntries
              .length}件)
          </button>
        {/if}
      </div>
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
