<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { AppSettings, TemplateValidation } from "../types";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  let libraryRoot = $state("");
  let directoryTemplate = $state("");
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

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      libraryRoot = settings.libraryRoot ?? "";
      directoryTemplate = settings.directoryTemplate ?? "";
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
      await invoke("set_directory_template", {
        template: directoryTemplate.trim(),
      });
      message = {
        type: "success",
        text: "ディレクトリテンプレートを保存しました",
      };
    } catch (e) {
      message = { type: "error", text: `保存に失敗しました: ${e}` };
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
    try {
      await invoke("validate_template", { template: trimmed });
      templateValidation = { valid: true, error: null };
      try {
        const preview = await invoke<string>("preview_template", {
          template: trimmed,
        });
        templatePreview = preview;
      } catch {
        templatePreview = null;
      }
    } catch (e) {
      templateValidation = { valid: false, error: String(e) };
      templatePreview = null;
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
          <code>{"{origin}"}</code>
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
    </div>

    {#if message}
      <p class="settings-message {message.type}">{message.text}</p>
    {/if}
  {/if}
</main>
