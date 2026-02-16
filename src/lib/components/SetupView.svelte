<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Library } from "../types";

  interface Props {
    onComplete: (library: Library) => void;
  }

  let { onComplete }: Props = $props();

  let step = $state<"welcome" | "select">("welcome");
  let selectedPath = $state<string | null>(null);
  let libraryName = $state("");
  let creating = $state(false);
  let error = $state<string | null>(null);

  async function selectDirectory() {
    const selected = await open({ directory: true });
    if (!selected) return;

    selectedPath = selected;
    libraryName = selected.split(/[/\\]/).pop() || selected;
    error = null;
  }

  async function createLibrary() {
    if (!selectedPath || !libraryName.trim()) return;

    creating = true;
    error = null;
    try {
      const lib = await invoke<Library>("create_library", {
        name: libraryName.trim(),
        path: selectedPath,
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
    <button class="setup-primary-btn" onclick={() => (step = "select")}>
      はじめる
    </button>
  {:else}
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
        <input
          class="setup-name-input"
          type="text"
          bind:value={libraryName}
        />
      </label>

      <button
        class="setup-primary-btn"
        disabled={!libraryName.trim() || creating}
        onclick={createLibrary}
      >
        {creating ? "作成中..." : "作成"}
      </button>
    {/if}

    {#if error}
      <p class="setup-error">{error}</p>
    {/if}
  {/if}
</div>
