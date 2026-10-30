<script>
  import { store } from './store.svelte.js';
</script>

<div class="wrap" role="status" aria-live="polite">
  {#if store.snack}
    {#key store.snack.id}
      <div class="snack body-m">
        <span class="msg">{store.snack.text}</span>
        <button class="act label-l" onclick={() => (store.snack = null)}>OK</button>
      </div>
    {/key}
  {/if}
</div>

<style>
  .wrap {
    position: fixed;
    left: 16px;
    right: 16px;
    bottom: calc(16px + var(--snack-offset, 0px));
    display: flex;
    justify-content: center;
    pointer-events: none;
    z-index: 20;
  }
  .snack {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 48px;
    max-width: 560px;
    width: 100%;
    padding: 6px 8px 6px 16px;
    background: var(--md-inverse-surface);
    color: var(--md-inverse-on-surface);
    border-radius: var(--shape-xs);
    box-shadow: 0 3px 6px rgb(0 0 0 / 0.3);
    animation: in var(--spatial-fast);
  }
  .msg { flex: 1; }
  .act {
    border: 0;
    background: none;
    color: var(--md-inverse-primary);
    height: 40px;
    padding: 0 12px;
    border-radius: var(--shape-full);
    cursor: pointer;
  }
  @media (max-width: 599px) { .wrap { bottom: 96px; } }
  @keyframes in { from { opacity: 0; transform: translateY(16px); } }
  @media (prefers-reduced-motion: reduce) { .snack { animation: none; } }
</style>
