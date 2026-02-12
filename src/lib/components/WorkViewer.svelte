<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { WorkDetail, FitMode, SlideshowMode } from "../types";

  interface Props {
    workId: number;
    workIds: number[];
    onBack: () => void;
    onNavigateWork: (workId: number) => void;
  }

  let { workId, workIds, onBack, onNavigateWork }: Props = $props();

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

  let slideshowPlaying = $state(false);
  let slideshowInterval = $state(5);
  let slideshowMode = $state<SlideshowMode>("page");
  let slideshowLoop = $state(false);
  let slideshowTick = $state(0);
  let isFullscreen = $state(false);
  let controlsVisible = $state(true);
  let controlsTimeoutId = $state<ReturnType<typeof setTimeout> | null>(null);
  let intervalInputFocused = $state(false);

  let imageUrl = $derived(`sharaku://localhost/view/${workId}/${currentPage}`);
  let pageCount = $derived(work?.pageCount ?? 1);

  let currentWorkIndex = $derived(workIds.indexOf(workId));
  let totalWorks = $derived(workIds.length);
  let hasMultipleWorks = $derived(totalWorks > 1 && currentWorkIndex >= 0);

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

  function navigateToWork(newWorkId: number) {
    currentPage = 0;
    onNavigateWork(newWorkId);
  }

  function navigatePrev() {
    if (currentPage > 0) {
      currentPage--;
    } else if (hasMultipleWorks && currentWorkIndex > 0) {
      navigateToWork(workIds[currentWorkIndex - 1]);
    } else if (slideshowLoop && hasMultipleWorks) {
      navigateToWork(workIds[totalWorks - 1]);
    }
  }

  function navigateNext() {
    if (currentPage < pageCount - 1) {
      currentPage++;
    } else if (hasMultipleWorks && currentWorkIndex < totalWorks - 1) {
      navigateToWork(workIds[currentWorkIndex + 1]);
    } else if (slideshowLoop && hasMultipleWorks) {
      navigateToWork(workIds[0]);
    }
  }

  function advanceSlideshow() {
    if (!work) return;
    if (slideshowMode === "page") {
      if (currentPage < pageCount - 1) {
        currentPage++;
      } else if (hasMultipleWorks && currentWorkIndex < totalWorks - 1) {
        navigateToWork(workIds[currentWorkIndex + 1]);
      } else if (slideshowLoop) {
        navigateToWork(workIds[0]);
      } else {
        slideshowPlaying = false;
      }
    } else {
      if (hasMultipleWorks && currentWorkIndex < totalWorks - 1) {
        navigateToWork(workIds[currentWorkIndex + 1]);
      } else if (slideshowLoop) {
        navigateToWork(workIds[0]);
      } else {
        slideshowPlaying = false;
      }
    }
  }

  function toggleSlideshow() {
    slideshowPlaying = !slideshowPlaying;
  }

  async function toggleFullscreen() {
    const win = getCurrentWindow();
    const current = await win.isFullscreen();
    await win.setFullscreen(!current);
    isFullscreen = !current;
  }

  function showControls() {
    controlsVisible = true;
    if (controlsTimeoutId) {
      clearTimeout(controlsTimeoutId);
      controlsTimeoutId = null;
    }
    if (isFullscreen && slideshowPlaying) {
      controlsTimeoutId = setTimeout(() => {
        controlsVisible = false;
        controlsTimeoutId = null;
      }, 3000);
    }
  }

  function handleMouseMove() {
    if (isFullscreen) {
      showControls();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (intervalInputFocused && e.key !== "Escape") return;

    switch (e.key) {
      case "Escape":
        if (isFullscreen) {
          toggleFullscreen();
          slideshowPlaying = false;
        } else {
          onBack();
        }
        break;
      case "ArrowLeft":
        navigatePrev();
        if (slideshowPlaying) slideshowTick++;
        break;
      case "ArrowRight":
        navigateNext();
        if (slideshowPlaying) slideshowTick++;
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
      case "s":
        fitMode = "screen";
        zoom = 1;
        break;
      case "f":
        toggleFullscreen();
        break;
      case "l":
        slideshowLoop = !slideshowLoop;
        break;
      case " ":
        e.preventDefault();
        toggleSlideshow();
        break;
    }
  }

  function handleIntervalInput(e: Event) {
    const value = parseFloat((e.target as HTMLInputElement).value);
    if (!isNaN(value) && value >= 1 && value <= 999) {
      slideshowInterval = value;
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
    work = null;
    error = null;
    currentPage = 0;
    loadWork();
  });

  $effect(() => {
    if (!containerEl) return;
    const observer = new ResizeObserver(() => updateContainerSize());
    observer.observe(containerEl);
    updateContainerSize();
    return () => observer.disconnect();
  });

  $effect(() => {
    if (!slideshowPlaying) return;
    void slideshowTick;
    const timer = setInterval(
      () => advanceSlideshow(),
      slideshowInterval * 1000,
    );
    return () => clearInterval(timer);
  });

  $effect(() => {
    if (!isFullscreen || !slideshowPlaying) {
      controlsVisible = true;
      if (controlsTimeoutId) {
        clearTimeout(controlsTimeoutId);
        controlsTimeoutId = null;
      }
    } else {
      showControls();
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="viewer-overlay" onmousemove={handleMouseMove}>
  <div
    class="viewer-header"
    class:viewer-controls-hidden={isFullscreen && !controlsVisible}
  >
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
        title="画面フィット (S)"
      >
        S
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

  <div
    class="viewer-footer"
    class:viewer-controls-hidden={isFullscreen && !controlsVisible}
  >
    <div class="viewer-slideshow-controls">
      <button
        class="viewer-control-btn"
        disabled={!slideshowLoop &&
          currentPage <= 0 &&
          (!hasMultipleWorks || currentWorkIndex <= 0)}
        onclick={() => {
          navigatePrev();
          if (slideshowPlaying) slideshowTick++;
        }}
        title="前へ"
      >
        ⏮
      </button>
      <button
        class="viewer-control-btn viewer-play-btn"
        onclick={toggleSlideshow}
        title={slideshowPlaying ? "停止 (Space)" : "再生 (Space)"}
      >
        {slideshowPlaying ? "⏸" : "▶"}
      </button>
      <button
        class="viewer-control-btn"
        disabled={!slideshowLoop &&
          currentPage >= pageCount - 1 &&
          (!hasMultipleWorks || currentWorkIndex >= totalWorks - 1)}
        onclick={() => {
          navigateNext();
          if (slideshowPlaying) slideshowTick++;
        }}
        title="次へ"
      >
        ⏭
      </button>

      <span class="viewer-slideshow-separator">|</span>

      <span class="viewer-interval-label">間隔</span>
      <input
        type="number"
        class="viewer-interval-input"
        value={slideshowInterval}
        min="1"
        max="999"
        onchange={handleIntervalInput}
        onfocus={() => (intervalInputFocused = true)}
        onblur={() => (intervalInputFocused = false)}
      />
      <span class="viewer-interval-label">秒</span>

      <span class="viewer-slideshow-separator">|</span>

      <label class="viewer-mode-label">
        <input
          type="radio"
          name="slideshow-mode"
          value="page"
          checked={slideshowMode === "page"}
          onchange={() => (slideshowMode = "page")}
        />
        ページ
      </label>
      <label class="viewer-mode-label">
        <input
          type="radio"
          name="slideshow-mode"
          value="work"
          checked={slideshowMode === "work"}
          onchange={() => (slideshowMode = "work")}
        />
        作品
      </label>

      <span class="viewer-slideshow-separator">|</span>

      <button
        class="viewer-interval-btn"
        class:active={slideshowLoop}
        onclick={() => (slideshowLoop = !slideshowLoop)}
        title="ループ (L)"
      >
        ループ
      </button>

      <span class="viewer-slideshow-separator">|</span>

      <button
        class="viewer-control-btn"
        onclick={toggleFullscreen}
        title="フルスクリーン (F)"
      >
        {isFullscreen ? "⛶" : "⛶"}
      </button>

      <span class="viewer-page-info">
        {currentPage + 1}/{pageCount}
        {#if hasMultipleWorks}
          (作品 {currentWorkIndex + 1}/{totalWorks})
        {/if}
      </span>
    </div>
  </div>
</div>
