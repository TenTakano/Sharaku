<script module lang="ts">
  /* eslint-disable @typescript-eslint/no-explicit-any */
  export const dragDropHandlers: {
    onDragOver?: (event: any) => void;
    onDragEnd?: (event: any) => void;
  } = {};
</script>

<script lang="ts">
  /* eslint-disable @typescript-eslint/no-explicit-any */
  let { children, onDragOver, onDragEnd }: any = $props();
  $effect(() => {
    dragDropHandlers.onDragOver = onDragOver;
    dragDropHandlers.onDragEnd = onDragEnd;
  });
</script>

{@render children?.()}
