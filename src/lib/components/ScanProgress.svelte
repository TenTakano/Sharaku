<script lang="ts">
  import type { ScanProgress } from "../types";

  interface Props {
    progress: ScanProgress | null;
  }

  let { progress }: Props = $props();
</script>

{#if progress}
  <div class="scan-progress">
    {#if progress.type === "started"}
      <p>{progress.total} 件のファイルを検出</p>
      <progress max={progress.total} value={0}></progress>
    {:else if progress.type === "processing"}
      <p>{progress.current} / {progress.total}: {progress.fileName}</p>
      <progress max={progress.total} value={progress.current}></progress>
    {:else if progress.type === "completed"}
      <p class="completed">完了: {progress.registered} 件登録, {progress.failed} 件失敗</p>
    {:else if progress.type === "error"}
      <p class="error">エラー: {progress.message}</p>
    {/if}
  </div>
{/if}
