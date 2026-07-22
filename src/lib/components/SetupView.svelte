<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Library, ResourceMode } from "../types";

  interface Props {
    onComplete: (library: Library) => void;
    onCancel?: () => void;
    initialStep?: "welcome" | "mode-select";
  }

  let { onComplete, onCancel, initialStep = "welcome" }: Props = $props();

  const TEMPLATE_PRESETS: string[] = [
    "{artist}/{title}",
    "{genre}/{artist}/{title}",
    "{type}/{artist}/{title}",
  ];

  let step = $state<
    "welcome" | "mode-select" | "select" | "template" | "name-only"
  >(initialStep);
  let selectedMode = $state<ResourceMode>("full");
  let selectedPath = $state<string | null>(null);
  let libraryName = $state("");
  let creating = $state(false);
  let error = $state<string | null>(null);

  let templateChoice = $state<"preset" | "custom">("preset");
  let selectedPreset = $state(TEMPLATE_PRESETS[0]);
  let customTemplate = $state("");
  let templateError = $state<string | null>(null);

  let selectedTemplate = $derived(
    templateChoice === "custom" ? customTemplate : selectedPreset,
  );

  function proceedFromModeSelect() {
    if (selectedMode === "full") {
      step = "select";
    } else {
      step = "name-only";
    }
  }

  function proceedToTemplate() {
    if (!selectedPath || !libraryName.trim()) return;
    step = "template";
    error = null;
  }

  async function validateCustomTemplate() {
    const trimmed = customTemplate.trim();
    if (!trimmed) {
      templateError = "テンプレートを入力してください";
      return;
    }
    try {
      await invoke("validate_template", { template: trimmed });
      templateError = null;
    } catch (e) {
      templateError = String(e);
    }
  }

  async function selectDirectory() {
    const selected = await open({ directory: true });
    if (!selected) return;

    selectedPath = selected;
    libraryName = selected.split(/[/\\]/).pop() || selected;
    error = null;
  }

  async function createLibraryFull() {
    if (!selectedPath || !libraryName.trim()) return;

    if (templateChoice === "custom") {
      await validateCustomTemplate();
      if (templateError) return;
    }

    creating = true;
    error = null;
    try {
      const lib = await invoke<Library>("create_library", {
        name: libraryName.trim(),
        path: selectedPath,
        resourceMode: "full",
        directoryTemplate: selectedTemplate,
      });
      onComplete(lib);
    } catch (e) {
      error = String(e);
    } finally {
      creating = false;
    }
  }

  async function createLibraryMetadataOnly() {
    if (!libraryName.trim()) return;

    creating = true;
    error = null;
    try {
      const lib = await invoke<Library>("create_library", {
        name: libraryName.trim(),
        path: null,
        resourceMode: "metadata_only",
      });
      onComplete(lib);
    } catch (e) {
      error = String(e);
    } finally {
      creating = false;
    }
  }
</script>

<div class="setup-container">
  {#if step === "welcome"}
    <h1 class="setup-title">Sharaku</h1>
    <p class="setup-subtitle">画像ライブラリ管理ツールへようこそ</p>
    <button class="setup-primary-btn" onclick={() => (step = "mode-select")}>
      はじめる
    </button>
  {:else if step === "mode-select"}
    <h2 class="setup-step-title">リソース管理モードを選択してください</h2>
    <p class="setup-step-description">
      ライブラリでのファイル管理方法を選択します。
    </p>

    <div class="setup-mode-select">
      <label class="radio-option">
        <input
          type="radio"
          name="setup-resource-mode"
          checked={selectedMode === "full"}
          onchange={() => (selectedMode = "full")}
        />
        <span class="radio-option-label">すべて管理</span>
        <span class="radio-option-desc">テンプレートに基づきファイルを配置</span
        >
      </label>
      <label class="radio-option">
        <input
          type="radio"
          name="setup-resource-mode"
          checked={selectedMode === "metadata_only"}
          onchange={() => (selectedMode = "metadata_only")}
        />
        <span class="radio-option-label">メタデータのみ管理</span>
        <span class="radio-option-desc"
          >ファイルを移動せず、メタデータのみ管理</span
        >
      </label>
    </div>

    <div class="setup-btn-row">
      <button class="setup-primary-btn" onclick={proceedFromModeSelect}>
        次へ
      </button>
      {#if onCancel}
        <button class="setup-secondary-btn" onclick={onCancel}>
          キャンセル
        </button>
      {/if}
    </div>
  {:else if step === "select"}
    <h2 class="setup-step-title">ライブラリのディレクトリを選択してください</h2>
    <p class="setup-step-description">
      画像ファイルが保存されているフォルダを選択します。
    </p>

    <button class="setup-select-btn" onclick={selectDirectory}>
      フォルダを選択
    </button>

    {#if selectedPath}
      <div class="setup-path-display">{selectedPath}</div>

      <label class="setup-name-label">
        ライブラリ名
        <input class="setup-name-input" type="text" bind:value={libraryName} />
      </label>

      <div class="setup-btn-row">
        <button
          class="setup-primary-btn"
          disabled={!libraryName.trim()}
          onclick={proceedToTemplate}
        >
          次へ
        </button>
        {#if onCancel}
          <button class="setup-secondary-btn" onclick={onCancel}>
            キャンセル
          </button>
        {/if}
      </div>
    {/if}

    {#if error}
      <p class="setup-error">{error}</p>
    {/if}
  {:else if step === "template"}
    <h2 class="setup-step-title">ディレクトリテンプレートを選択してください</h2>
    <p class="setup-step-description">
      作品取り込み時のフォルダ配置パターンを指定します。
    </p>

    <div class="setup-template-options">
      {#each TEMPLATE_PRESETS as preset (preset)}
        <label class="setup-template-option">
          <input
            type="radio"
            name="setup-template"
            checked={templateChoice === "preset" && selectedPreset === preset}
            onchange={() => {
              templateChoice = "preset";
              selectedPreset = preset;
              templateError = null;
            }}
          />
          <code class="setup-template-code">{preset}</code>
        </label>
      {/each}
      <label class="setup-template-option">
        <input
          type="radio"
          name="setup-template"
          checked={templateChoice === "custom"}
          onchange={() => {
            templateChoice = "custom";
            templateError = null;
          }}
        />
        <span>カスタム</span>
      </label>
    </div>

    {#if templateChoice === "custom"}
      <div class="setup-template-custom">
        <input
          type="text"
          class="setup-name-input"
          class:settings-input-error={templateError !== null}
          bind:value={customTemplate}
          oninput={validateCustomTemplate}
          placeholder="{'{artist}'}/{'{title}'}"
        />
        <p class="setup-template-placeholders">
          使用可能: <code>{"{title}"}</code>, <code>{"{artist}"}</code>,
          <code>{"{year}"}</code>, <code>{"{genre}"}</code>,
          <code>{"{circle}"}</code>, <code>{"{origin}"}</code>,
          <code>{"{type}"}</code>
        </p>
      </div>
    {/if}

    {#if templateError}
      <p class="setup-error">{templateError}</p>
    {/if}

    {#if error}
      <p class="setup-error">{error}</p>
    {/if}

    <div class="setup-btn-row">
      <button
        class="setup-primary-btn"
        disabled={creating ||
          (templateChoice === "custom" &&
            (!customTemplate.trim() || templateError !== null))}
        onclick={createLibraryFull}
      >
        {creating ? "作成中..." : "作成"}
      </button>
      <button class="setup-secondary-btn" onclick={() => (step = "select")}>
        戻る
      </button>
    </div>
  {:else if step === "name-only"}
    <h2 class="setup-step-title">ライブラリ名を入力してください</h2>
    <p class="setup-step-description">
      メタデータのみ管理するライブラリを作成します。
    </p>

    <label class="setup-name-label">
      ライブラリ名
      <input
        class="setup-name-input"
        type="text"
        bind:value={libraryName}
        placeholder="マイライブラリ"
      />
    </label>

    <div class="setup-btn-row">
      <button
        class="setup-primary-btn"
        disabled={!libraryName.trim() || creating}
        onclick={createLibraryMetadataOnly}
      >
        {creating ? "作成中..." : "作成"}
      </button>
      {#if onCancel}
        <button class="setup-secondary-btn" onclick={onCancel}>
          キャンセル
        </button>
      {/if}
    </div>

    {#if error}
      <p class="setup-error">{error}</p>
    {/if}
  {/if}
</div>
