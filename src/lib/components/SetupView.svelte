<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Library } from "../types";

  interface Props {
    onComplete: (library: Library) => void;
    onCancel?: () => void;
    initialStep?: "welcome" | "select";
  }

  let { onComplete, onCancel, initialStep = "welcome" }: Props = $props();

  // svelte-ignore state_referenced_locally
  let step = $state<"welcome" | "select">(initialStep);
  let selectedPath = $state<string | null>(null);
  let pathSkipped = $state(false);
  let libraryName = $state("");
  let creating = $state(false);
  let error = $state<string | null>(null);

  let showNameInput = $derived(selectedPath !== null || pathSkipped);

  async function selectDirectory() {
    const selected = await open({ directory: true });
    if (!selected) return;

    selectedPath = selected;
    pathSkipped = false;
    libraryName = selected.split(/[/\\]/).pop() || selected;
    error = null;
  }

  function skipDirectory() {
    pathSkipped = true;
    error = null;
  }

  async function createLibrary() {
    if (!libraryName.trim()) return;

    creating = true;
    error = null;
    try {
      const lib = await invoke<Library>("create_library", {
        name: libraryName.trim(),
        path: pathSkipped ? null : selectedPath,
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
  {:else if step === "select"}
    <h2 class="setup-step-title">ライブラリのディレクトリを選択してください</h2>
    <p class="setup-step-description">
      画像ファイルが保存されているフォルダを選択します。ディレクトリを設定せずに続けることもできます。
    </p>

    <button class="setup-select-btn" onclick={selectDirectory}>
      フォルダを選択
    </button>

    {#if !showNameInput}
      <div class="setup-btn-row">
        <button class="setup-secondary-btn" onclick={skipDirectory}>
          フォルダを設定せずに続ける
        </button>
      </div>
    {/if}

    {#if selectedPath}
      <div class="setup-path-display">{selectedPath}</div>
    {/if}

    {#if showNameInput}
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
          onclick={createLibrary}
        >
          {creating ? "作成中..." : "作成"}
        </button>
        {#if onCancel}
          <button class="setup-secondary-btn" onclick={onCancel}>
            キャンセル
          </button>
        {/if}
      </div>
    {:else if onCancel}
      <div class="setup-btn-row">
        <button class="setup-secondary-btn" onclick={onCancel}>
          キャンセル
        </button>
      </div>
    {/if}

    {#if error}
      <p class="setup-error">{error}</p>
    {/if}
  {/if}
</div>
