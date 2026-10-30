<script>
  // Basic modal dialog: 28dp radius, 24dp padding, 280–560dp wide, actions trailing.
  let { open = $bindable(false), title, children, actions, onclose } = $props();
  let el;

  $effect(() => {
    if (open && !el.open) el.showModal();
    if (!open && el.open) el.close();
  });

  function close() {
    open = false;
    onclose?.();
  }
</script>

<dialog bind:this={el} oncancel={(e) => { e.preventDefault(); close(); }} onclick={(e) => e.target === el && close()}>
  <div class="body">
    <h2 class="headline-s">{title}</h2>
    <div class="content body-m">{@render children?.()}</div>
    <div class="actions">{@render actions?.()}</div>
  </div>
</dialog>

<style>
  dialog {
    border: 0;
    padding: 0;
    width: min(560px, calc(100vw - 32px));
    min-width: min(280px, 100vw);
    max-height: calc(100vh - 64px);
    border-radius: var(--shape-xl);
    background: var(--md-surface-container-high);
    --tf-bg: var(--md-surface-container-high);
    color: var(--md-on-surface);
    box-shadow: 0 6px 16px rgb(0 0 0 / 0.3);
    overflow: hidden;
  }
  dialog::backdrop { background: rgb(0 0 0 / 0.32); }
  dialog[open] { animation: enter var(--spatial-default); }
  dialog[open]::backdrop { animation: fade var(--effects-default); }
  .body { display: grid; gap: 16px; padding: 24px; max-height: inherit; grid-template-rows: auto minmax(0, 1fr) auto; }
  .content { overflow: auto; color: var(--md-on-surface-variant); }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }
  @keyframes enter { from { opacity: 0; transform: scale(0.9) translateY(16px); } }
  @keyframes fade { from { opacity: 0; } }
  @media (prefers-reduced-motion: reduce) {
    dialog[open], dialog[open]::backdrop { animation: none; }
  }
</style>
