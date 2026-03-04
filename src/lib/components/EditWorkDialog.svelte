<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { addToast } from "../stores/toast.svelte";
  import type { WorkDetail } from "../types";

  interface Props {
    work: WorkDetail;
    onClose: () => void;
    onUpdated: (workId: number) => void;
  }

  let { work, onClose, onUpdated }: Props = $props();

  let title = $state(work.title);
  let artist = $state(work.artist ?? "");
  let year = $state(work.year?.toString() ?? "");
  let genre = $state(work.genre ?? "");
  let circle = $state(work.circle ?? "");
  let origin = $state(work.origin ?? "");
  let saving = $state(false);

  let canSave = $derived(title.trim().length > 0 && !saving);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      onClose();
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  async function handleSave() {
    if (!canSave) return;
    saving = true;
    try {
      const parsedYear = year.trim() ? parseInt(year.trim(), 10) : null;
      if (parsedYear !== null && isNaN(parsedYear)) {
        addToast("error", "年は数値で入力してください");
        saving = false;
        return;
      }
      await invoke("update_work", {
        id: work.id,
        title: title.trim(),
        artist: artist.trim() || null,
        year: parsedYear,
        genre: genre.trim() || null,
        circle: circle.trim() || null,
        origin: origin.trim() || null,
      });
      addToast("success", "メタデータを更新しました");
      onUpdated(work.id);
      onClose();
    } catch (e) {
      addToast("error", `更新に失敗しました: ${e}`);
    } finally {
      saving = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="edit-work-overlay" onmousedown={handleOverlayClick}>
  <div class="edit-work-dialog">
    <h3 class="edit-work-dialog-title">メタデータを編集</h3>
    <div class="edit-work-form">
      <label class="edit-work-label">
        タイトル <span class="edit-work-required">*</span>
        <input
          class="settings-input"
          type="text"
          bind:value={title}
          placeholder="タイトル"
        />
      </label>
      <label class="edit-work-label">
        アーティスト
        <input
          class="settings-input"
          type="text"
          bind:value={artist}
          placeholder="アーティスト"
        />
      </label>
      <label class="edit-work-label">
        年
        <input
          class="settings-input"
          type="number"
          bind:value={year}
          placeholder="年"
        />
      </label>
      <label class="edit-work-label">
        ジャンル
        <input
          class="settings-input"
          type="text"
          bind:value={genre}
          placeholder="ジャンル"
        />
      </label>
      <label class="edit-work-label">
        サークル
        <input
          class="settings-input"
          type="text"
          bind:value={circle}
          placeholder="サークル"
        />
      </label>
      <label class="edit-work-label">
        出典
        <input
          class="settings-input"
          type="text"
          bind:value={origin}
          placeholder="出典"
        />
      </label>
    </div>
    <div class="edit-work-actions">
      <button class="edit-work-cancel" onclick={onClose}>キャンセル</button>
      <button class="edit-work-save" disabled={!canSave} onclick={handleSave}>
        {saving ? "保存中..." : "保存"}
      </button>
    </div>
  </div>
</div>
