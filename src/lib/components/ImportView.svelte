<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { addToast } from "../stores/toast.svelte";
  import type {
    ImportKind,
    ImportRequest,
    EnqueueResult,
    ParsedMetadata,
  } from "../types";

  interface Props {
    initialSourcePath?: string;
    initialKind?: ImportKind;
    onBack: () => void;
  }

  let { initialSourcePath, initialKind = "folder", onBack }: Props = $props();

  type Step = "select" | "metadata";

  const IMAGE_EXTENSIONS = ["jpg", "jpeg", "png", "gif", "webp", "bmp"];

  let step = $state<Step>("select");
  // svelte-ignore state_referenced_locally
  let kind = $state<ImportKind>(initialKind);
  let sourcePath = $state("");
  let title = $state("");
  let artist = $state("");
  let year = $state("");
  let genre = $state("");
  let circle = $state("");
  let origin = $state("");
  let submitting = $state(false);

  function basenameOf(path: string): string {
    const sep = path.includes("\\") ? "\\" : "/";
    return path.split(sep).pop() ?? path;
  }

  function stripExtension(fileName: string): string {
    const dot = fileName.lastIndexOf(".");
    return dot > 0 ? fileName.slice(0, dot) : fileName;
  }

  async function resolveMetadataFromPath(
    path: string,
  ): Promise<{ title: string; artist: string }> {
    const base = basenameOf(path);
    const nameForParse = kind === "image" ? stripExtension(base) : base;

    try {
      const parsed = await invoke<ParsedMetadata>("parse_folder_name", {
        folderName: nameForParse,
      });
      return { title: parsed.title, artist: parsed.artist ?? "" };
    } catch {
      return { title: nameForParse, artist: "" };
    }
  }

  async function selectSource() {
    const selected =
      kind === "folder"
        ? await open({ directory: true })
        : await open({
            directory: false,
            multiple: false,
            filters: [{ name: "画像", extensions: IMAGE_EXTENSIONS }],
          });
    if (typeof selected !== "string") return;
    await loadFromPath(selected);
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

  async function executeImport() {
    submitting = true;
    try {
      const request: ImportRequest = {
        sourcePath,
        ...buildMetadata(),
        kind,
      };
      await invoke<EnqueueResult>("enqueue_import", { requests: [request] });
      onBack();
    } catch (e) {
      addToast("error", String(e));
    } finally {
      submitting = false;
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
  }

  async function loadFromPath(path: string) {
    sourcePath = path;
    const resolved = await resolveMetadataFromPath(path);
    title = resolved.title;
    artist = resolved.artist;

    step = "metadata";
  }

  $effect(() => {
    if (initialSourcePath) {
      loadFromPath(initialSourcePath);
    }
  });
</script>

{#if step === "select"}
  <div class="import-content">
    <section class="import-section">
      <h2>取り込み対象を選択</h2>
      <div class="import-kind-select">
        <label class="import-mode-option">
          <input type="radio" bind:group={kind} value="folder" />
          フォルダ（複数画像）
        </label>
        <label class="import-mode-option">
          <input type="radio" bind:group={kind} value="image" />
          画像ファイル（1枚）
        </label>
      </div>
      <p class="import-description">
        {kind === "folder"
          ? "取り込む画像フォルダを選択してください。"
          : "取り込む画像ファイルを選択してください。"}
      </p>
      <button class="import-select-btn" onclick={selectSource}>
        {kind === "folder" ? "フォルダを選択..." : "画像を選択..."}
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
              placeholder="出典"
            />
          </div>
        </div>
      </div>

      <div class="template-preview">
        <span class="template-preview-label">元の場所に登録されます</span>
      </div>

      <div class="import-actions">
        <button class="settings-back-btn" onclick={resetForm}> ← 戻る </button>
        <button
          class="import-execute-btn"
          onclick={executeImport}
          disabled={!title.trim() || submitting}
        >
          取り込み実行
        </button>
      </div>
    </section>
  </div>
{/if}
