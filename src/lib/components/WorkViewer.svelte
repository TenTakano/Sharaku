<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { WorkDetail, FitMode } from "../types";

  interface Props {
    workId: number;
    onBack: () => void;
  }

  let { workId, onBack }: Props = $props();

  let work = $state<WorkDetail | null>(null);
  let error = $state<string | null>(null);
  let fitMode = $state<FitMode>("screen");
  let zoom = $state(1);
  let currentPage = $state(0);
  let naturalWidth = $state(0);
  let naturalHeight = $state(0);
  let containerEl = $state<HTMLDivElement | null>(null);
  let containerWidth = $state(0);
  let containerHeight = $state(0);

  let imageUrl = $derived(`sharaku://localhost/view/${workId}/${currentPage}`);
  let pageCount = $derived(work?.pageCount ?? 1);

  let displayStyle = $derived.by(() => {
    if (
      !naturalWidth ||
      !naturalHeight ||
      !containerWidth ||
      !containerHeight
    ) {
      return "max-width: 100%; max-height: 100%; object-fit: contain;";
    }

    let baseWidth: number;
    let baseHeight: number;

    switch (fitMode) {
      case "width":
        baseWidth = containerWidth;
        baseHeight = (containerWidth / naturalWidth) * naturalHeight;
        break;
      case "height":
        baseHeight = containerHeight;
        baseWidth = (containerHeight / naturalHeight) * naturalWidth;
        break;
      case "screen":
      default: {
        const scaleW = containerWidth / naturalWidth;
        const scaleH = containerHeight / naturalHeight;
        const scale = Math.min(scaleW, scaleH);
        baseWidth = naturalWidth * scale;
        baseHeight = naturalHeight * scale;
        break;
      }
    }

    const w = baseWidth * zoom;
    const h = baseHeight * zoom;
    return `width: ${w}px; height: ${h}px;`;
  });

  async function loadWork() {
    try {
      work = await invoke("get_work", { workId });
    } catch (e) {
      error = String(e);
    }
  }

  function handleImageLoad(e: Event) {
    const img = e.target as HTMLImageElement;
    naturalWidth = img.naturalWidth;
    naturalHeight = img.naturalHeight;
  }

  function handleWheel(e: WheelEvent) {
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.1 : 0.1;
    zoom = Math.min(4, Math.max(0.25, zoom + delta));
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey || e.metaKey || e.altKey) return;

    switch (e.key) {
      case "Escape":
        onBack();
        break;
      case "ArrowLeft":
        if (currentPage > 0) currentPage--;
        break;
      case "ArrowRight":
        if (currentPage < pageCount - 1) currentPage++;
        break;
      case "+":
      case "=":
        zoom = Math.min(4, zoom + 0.25);
        break;
      case "-":
        zoom = Math.max(0.25, zoom - 0.25);
        break;
      case "0":
        zoom = 1;
        break;
      case "w":
        fitMode = "width";
        zoom = 1;
        break;
      case "h":
        fitMode = "height";
        zoom = 1;
        break;
      case "f":
        fitMode = "screen";
        zoom = 1;
        break;
    }
  }

  function updateContainerSize() {
    if (containerEl) {
      containerWidth = containerEl.clientWidth;
      containerHeight = containerEl.clientHeight;
    }
  }

  $effect(() => {
    void workId;
    loadWork();
  });

  $effect(() => {
    if (!containerEl) return;
    const observer = new ResizeObserver(() => updateContainerSize());
    observer.observe(containerEl);
    updateContainerSize();
    return () => observer.disconnect();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="viewer-overlay">
  <div class="viewer-header">
    <button class="viewer-back-btn" onclick={onBack}>← 戻る</button>
    {#if work}
      <span class="viewer-title">{work.title}</span>
    {/if}
    <div class="viewer-toolbar">
      <button
        class="viewer-fit-btn"
        class:active={fitMode === "width"}
        onclick={() => {
          fitMode = "width";
          zoom = 1;
        }}
        title="幅フィット (W)"
      >
        W
      </button>
      <button
        class="viewer-fit-btn"
        class:active={fitMode === "height"}
        onclick={() => {
          fitMode = "height";
          zoom = 1;
        }}
        title="高さフィット (H)"
      >
        H
      </button>
      <button
        class="viewer-fit-btn"
        class:active={fitMode === "screen"}
        onclick={() => {
          fitMode = "screen";
          zoom = 1;
        }}
        title="画面フィット (F)"
      >
        F
      </button>
      <span class="viewer-zoom-label">{Math.round(zoom * 100)}%</span>
    </div>
  </div>

  <div class="viewer-content" bind:this={containerEl} onwheel={handleWheel}>
    {#if error}
      <p class="viewer-error">{error}</p>
    {:else if work}
      <div class="viewer-centering">
        <img
          class="viewer-image"
          src={imageUrl}
          alt={work.title}
          style={displayStyle}
          onload={handleImageLoad}
        />
      </div>
    {:else}
      <p class="viewer-loading">Loading...</p>
    {/if}
  </div>

  <div class="viewer-footer">
    <button
      class="viewer-page-btn"
      disabled={currentPage <= 0}
      onclick={() => currentPage--}
    >
      ◀
    </button>
    <span class="viewer-page-info">{currentPage + 1} / {pageCount}</span>
    <button
      class="viewer-page-btn"
      disabled={currentPage >= pageCount - 1}
      onclick={() => currentPage++}
    >
      ▶
    </button>
  </div>
</div>
