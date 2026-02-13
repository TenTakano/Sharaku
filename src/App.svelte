<script lang="ts">
  import ScanButton from "./lib/components/ScanButton.svelte";
  import ScanProgressBar from "./lib/components/ScanProgress.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import WorkGrid from "./lib/components/WorkGrid.svelte";
  import WorkViewer from "./lib/components/WorkViewer.svelte";
  import type { ScanProgress } from "./lib/types";

  let progress = $state<ScanProgress | null>(null);
  let reloadTrigger = $state(0);
  let currentView = $state<"library" | "viewer" | "settings">("library");
  let selectedWorkId = $state<number | null>(null);
  let workIds = $state<number[]>([]);

  function handleProgress(p: ScanProgress) {
    progress = p;
  }

  function handleComplete() {
    reloadTrigger++;
  }

  function handleSelectWork(workId: number) {
    selectedWorkId = workId;
    currentView = "viewer";
  }

  function handleWorksLoaded(ids: number[]) {
    workIds = ids;
  }

  function handleNavigateWork(workId: number) {
    selectedWorkId = workId;
  }

  function handleBackToLibrary() {
    currentView = "library";
    selectedWorkId = null;
  }
</script>

{#if currentView === "viewer" && selectedWorkId !== null}
  <WorkViewer
    workId={selectedWorkId}
    {workIds}
    onBack={handleBackToLibrary}
    onNavigateWork={handleNavigateWork}
  />
{:else if currentView === "settings"}
  <SettingsView onBack={handleBackToLibrary} />
{:else}
  <main class="container">
    <div class="app-header">
      <h1>Sharaku</h1>
      <ScanButton onProgress={handleProgress} onComplete={handleComplete} />
      <button
        class="settings-btn"
        onclick={() => (currentView = "settings")}
        title="設定"
      >
        ⚙
      </button>
    </div>
    <ScanProgressBar {progress} />
    <WorkGrid
      {reloadTrigger}
      onSelectWork={handleSelectWork}
      onWorksLoaded={handleWorksLoaded}
    />
  </main>
{/if}
