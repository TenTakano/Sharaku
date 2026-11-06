<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { addToast } from "../stores/toast.svelte";
  import type { AppSettings, DeleteFileAction, ViewKind } from "../types";

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

  let deleteFileAction = $state<DeleteFileAction>("ask");
  let loading = $state(true);

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      deleteFileAction = settings.deleteFileAction;
    } catch (e) {
      addToast("error", `設定の読み込みに失敗しました: ${e}`);
    } finally {
      loading = false;
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
        <h2>削除時のファイル処理</h2>
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
            />
            <span class="radio-option-label">合わせて削除する</span>
            <span class="radio-option-desc">ローカルファイルを完全に削除</span>
          </label>
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
