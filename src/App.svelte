<script lang="ts">
  import BulkImportView from "./lib/components/BulkImportView.svelte";
  import ImportView from "./lib/components/ImportView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import WorkGrid from "./lib/components/WorkGrid.svelte";
  import WorkViewer from "./lib/components/WorkViewer.svelte";

  let reloadTrigger = $state(0);
  let currentView = $state<
    "library" | "viewer" | "settings" | "import" | "bulk-import"
  >("library");
  let selectedWorkId = $state<number | null>(null);
  let workIds = $state<number[]>([]);

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
{:else if currentView === "import"}
  <ImportView onBack={handleBackToLibrary} onImported={() => reloadTrigger++} />
{:else if currentView === "bulk-import"}
  <BulkImportView
    onBack={handleBackToLibrary}
    onImported={() => reloadTrigger++}
  />
{:else}
  <main class="container">
    <div class="app-header">
      <h1>Sharaku</h1>
      <button
        class="import-header-btn"
        onclick={() => (currentView = "import")}
      >
        + 取り込み
      </button>
      <button
        class="import-header-btn"
        onclick={() => (currentView = "bulk-import")}
      >
        一括取り込み
      </button>
      <button
        class="settings-btn"
        onclick={() => (currentView = "settings")}
        title="設定"
      >
        ⚙
      </button>
    </div>
    <WorkGrid
      {reloadTrigger}
      onSelectWork={handleSelectWork}
      onWorksLoaded={handleWorksLoaded}
    />
  </main>
{/if}
