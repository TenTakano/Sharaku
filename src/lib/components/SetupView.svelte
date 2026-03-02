<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Library, ResourceMode } from "../types";

  interface Props {
    onComplete: (library: Library) => void;
  }

  let { onComplete }: Props = $props();

  let step = $state<"welcome" | "mode-select" | "select" | "name-only">(
    "welcome",
  );
  let selectedMode = $state<ResourceMode>("full");
  let selectedPath = $state<string | null>(null);
  let libraryName = $state("");
  let creating = $state(false);
  let error = $state<string | null>(null);

  function proceedFromModeSelect() {
    if (selectedMode === "full") {
      step = "select";
    } else {
      step = "name-only";
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

    creating = true;
    error = null;
    try {
      const lib = await invoke<Library>("create_library", {
        name: libraryName.trim(),
        path: selectedPath,
        resourceMode: "full",
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
      <label class="resource-mode-option">
        <input
          type="radio"
          name="setup-resource-mode"
          checked={selectedMode === "full"}
          onchange={() => (selectedMode = "full")}
        />
        <span class="resource-mode-label">すべて管理</span>
        <span class="resource-mode-desc"
          >テンプレートに基づきファイルを配置</span
        >
      </label>
      <label class="resource-mode-option">
        <input
          type="radio"
          name="setup-resource-mode"
          checked={selectedMode === "metadata_only"}
          onchange={() => (selectedMode = "metadata_only")}
        />
        <span class="resource-mode-label">メタデータのみ管理</span>
        <span class="resource-mode-desc"
          >ファイルを移動せず、メタデータのみ管理</span
        >
      </label>
    </div>

    <button class="setup-primary-btn" onclick={proceedFromModeSelect}>
      次へ
    </button>
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

      <button
        class="setup-primary-btn"
        disabled={!libraryName.trim() || creating}
        onclick={createLibraryFull}
      >
        {creating ? "作成中..." : "作成"}
      </button>
    {/if}

    {#if error}
      <p class="setup-error">{error}</p>
    {/if}
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

    <button
      class="setup-primary-btn"
      disabled={!libraryName.trim() || creating}
      onclick={createLibraryMetadataOnly}
    >
      {creating ? "作成中..." : "作成"}
    </button>

    {#if error}
      <p class="setup-error">{error}</p>
    {/if}
  {/if}
</div>
