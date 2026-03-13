<script lang="ts">
  interface Props {
    title: string;
    message: string;
    confirmLabel: string;
    danger?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    title,
    message,
    confirmLabel,
    danger = false,
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      onCancel();
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="confirm-dialog-overlay" onmousedown={handleOverlayClick}>
  <div class="confirm-dialog">
    <h3 class="confirm-dialog-title">{title}</h3>
    <p class="confirm-dialog-message">{message}</p>
    <div class="confirm-dialog-actions">
      <button class="confirm-dialog-cancel" onclick={onCancel}>
        キャンセル
      </button>
      <button
        class="confirm-dialog-confirm"
        class:confirm-dialog-danger={danger}
        onclick={onConfirm}
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
