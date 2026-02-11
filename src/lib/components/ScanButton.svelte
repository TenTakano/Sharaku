<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { ScanProgress } from "../types";

  interface Props {
    onProgress: (progress: ScanProgress) => void;
    onComplete: () => void;
  }

  let { onProgress, onComplete }: Props = $props();
  let scanning = $state(false);

  async function handleScan() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;

    scanning = true;
    const channel = new Channel<ScanProgress>();
    channel.onmessage = (progress) => {
      onProgress(progress);
      if (progress.type === "completed" || progress.type === "error") {
        scanning = false;
        onComplete();
      }
    };

    try {
      await invoke("scan_library", { rootPath: selected, onProgress: channel });
    } catch (e) {
      scanning = false;
      onProgress({ type: "error", message: String(e) });
    }
  }
</script>

<button class="scan-button" onclick={handleScan} disabled={scanning}>
  {scanning ? "スキャン中..." : "フォルダをスキャン"}
</button>
