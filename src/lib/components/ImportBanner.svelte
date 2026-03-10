<script lang="ts">
  import {
    getJobs,
    dismissJob,
    type ImportJobState,
  } from "../stores/importQueue.svelte";

  let visibleJobs = $derived(getJobs().filter((j) => j.status !== "queued"));

  function statusClass(job: ImportJobState): string {
    if (job.status === "running") return "import-banner-running";
    if (job.status === "completed" && job.failed > 0)
      return "import-banner-error";
    if (job.status === "completed") return "import-banner-success";
    return "import-banner-error";
  }
</script>

{#each visibleJobs as job (job.jobId)}
  <div class="import-banner {statusClass(job)}">
    {#if job.status === "running"}
      <div class="import-banner-text">
        {#if job.total > 1}
          取り込み中: {job.currentTitle} ({job.current}/{job.total})
        {:else}
          取り込み中: {job.currentTitle}
        {/if}
      </div>
      {#if job.total > 1}
        <progress
          class="import-banner-progress"
          max={job.total}
          value={job.current}
        ></progress>
      {/if}
    {:else if job.status === "completed"}
      <div class="import-banner-text">
        {#if job.failed > 0}
          取り込み完了: {job.succeeded}件成功 / {job.failed}件失敗
        {:else}
          取り込み完了: {job.succeeded}件成功
        {/if}
      </div>
      {#if job.errors.length > 0}
        <ul class="import-banner-errors">
          {#each job.errors as err, i (i)}
            <li><strong>{err.title}</strong>: {err.message}</li>
          {/each}
        </ul>
      {/if}
      <button class="toast-close" onclick={() => dismissJob(job.jobId)}>
        &times;
      </button>
    {:else}
      <div class="import-banner-text">取り込みに失敗しました</div>
      <button class="toast-close" onclick={() => dismissJob(job.jobId)}>
        &times;
      </button>
    {/if}
  </div>
{/each}
