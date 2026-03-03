<script lang="ts">
  import { getToasts, removeToast } from "../stores/toast.svelte";
</script>

{#each getToasts() as toast (toast.id)}
  <div class="toast toast-{toast.type}">
    <span class="toast-message">{toast.message}</span>
    <button class="toast-close" onclick={() => removeToast(toast.id)}>×</button>
  </div>
{/each}
