<script lang="ts">
  import ScanButton from "./lib/components/ScanButton.svelte";
  import ScanProgressBar from "./lib/components/ScanProgress.svelte";
  import WorkGrid from "./lib/components/WorkGrid.svelte";
  import type { ScanProgress } from "./lib/types";

  let progress = $state<ScanProgress | null>(null);
  let reloadTrigger = $state(0);

  function handleProgress(p: ScanProgress) {
    progress = p;
  }

  function handleComplete() {
    reloadTrigger++;
  }
</script>

<main class="container">
  <h1>Sharaku</h1>
  <div class="controls">
    <ScanButton onProgress={handleProgress} onComplete={handleComplete} />
  </div>
  <ScanProgressBar {progress} />
  <WorkGrid {reloadTrigger} />
</main>
