<script>
  import { theme, ACCENTS, isDark, toggleMode, setAccent } from './theme.svelte.js';
  import Icon from './Icon.svelte';

  let open = $state(false);
  const dark = $derived(isDark());

  function onwindow(e) {
    if (open && !e.target.closest?.('.appearance')) open = false;
  }
</script>

<svelte:window onclick={onwindow} onkeydown={(e) => e.key === 'Escape' && (open = false)} />

<div class="appearance">
  <button class="icon-btn" aria-label="Apparence" aria-haspopup="menu" aria-expanded={open} title="Apparence (thème et couleur)" onclick={() => (open = !open)}>
    <Icon name="palette" />
  </button>
  {#if open}
    <div class="menu" role="menu">
      <button role="menuitem" class="item" onclick={toggleMode}>
        <Icon name={dark ? 'light_mode' : 'dark_mode'} />
        <span class="body-m">{dark ? 'Thème clair' : 'Thème sombre'}</span>
      </button>
      <div class="sep"></div>
      {#each ACCENTS as a}
        <button role="menuitemradio" aria-checked={theme.accent === a.id} class="item" class:on={theme.accent === a.id} onclick={() => setAccent(a.id)}>
          <i class="dot" style="background:{a.seed}"></i>
          <span class="body-m">{a.name}</span>
          {#if theme.accent === a.id}<Icon name="check" />{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .appearance { position: fixed; top: 12px; right: 12px; z-index: 20; }
  .menu {
    position: absolute;
    top: 52px;
    right: 0;
    min-width: 200px;
    padding: 8px 0;
    border-radius: var(--shape-xs);
    background: var(--md-surface-container);
    box-shadow: 0 2px 6px 2px rgb(0 0 0 / 0.15), 0 1px 2px rgb(0 0 0 / 0.3);
    transform-origin: top right;
    animation: pop var(--effects-fast);
  }
  @keyframes pop { from { opacity: 0; transform: scale(0.9); } }
  .item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    min-height: 48px;
    padding: 0 16px 0 12px;
    border: 0;
    border-radius: 0;
    background: none;
    color: var(--md-on-surface);
    text-align: left;
  }
  .item:hover { background: color-mix(in srgb, var(--md-on-surface) 8%, transparent); }
  .item span { flex: 1; }
  .item.on span { font-weight: 700; }
  .item :global(svg) { width: 24px; height: 24px; flex: none; }
  .item.on :global(svg) { color: var(--md-primary); }
  .dot { width: 24px; height: 24px; border-radius: 50%; flex: none; box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--md-on-surface) 15%, transparent); }
  .sep { height: 1px; margin: 8px 0; background: var(--md-outline-variant); }
  @media (prefers-reduced-motion: reduce) { .menu { animation: none; } }
</style>
