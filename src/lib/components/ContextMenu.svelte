<script lang="ts">
  interface MenuItem {
    label: string;
    action: () => void;
  }

  interface Props {
    x: number;
    y: number;
    items: MenuItem[];
    onClose: () => void;
  }

  let { x, y, items, onClose }: Props = $props();

  function handleWindowClick() {
    onClose();
  }
</script>

<svelte:window onclick={handleWindowClick} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="context-menu"
  style="left: {x}px; top: {y}px;"
  oncontextmenu={(e) => e.preventDefault()}
>
  {#each items as item, i (i)}
    <button
      class="context-menu-item"
      onclick={(e) => {
        e.stopPropagation();
        item.action();
      }}
    >
      {item.label}
    </button>
  {/each}
</div>
