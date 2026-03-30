<script lang="ts">
  /* eslint-disable @typescript-eslint/no-explicit-any */
  let {
    data = [],
    children,
  }: { data: any[]; children: any; [key: string]: any } = $props();
</script>

{#each data as item, i (i)}
  {@render children(item, i)}
{/each}
