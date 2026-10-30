<script>
  import { onMount } from 'svelte';
  import { store, start } from './lib/store.svelte.js';
  import NewTask from './lib/NewTask.svelte';
  import Jobs from './lib/Jobs.svelte';
  import History from './lib/History.svelte';
  import Luts from './lib/Luts.svelte';
  import Icon from './lib/Icon.svelte';
  import Snackbar from './lib/Snackbar.svelte';

  let tab = $state('new');
  onMount(start);

  const active = $derived(store.jobs.filter((j) => !['done', 'failed', 'cancelled'].includes(j.status)));
  const items = [
    ['new', 'Nouvelle', 'add'],
    ['jobs', 'Tâches', 'movie'],
    ['history', 'Historique', 'history'],
    ['luts', 'LUT', 'tune'],
  ];
</script>

<div class="shell">
  <nav class="nav" aria-label="Navigation principale">
    <div class="brand" title={store.connected ? 'Connecté au serveur' : 'Déconnecté'}>
      <span class="status" class:off={!store.connected}></span>
    </div>
    {#each items as [id, label, icon]}
      <button class="nav-item" aria-current={tab === id ? 'page' : undefined} onclick={() => (tab = id)}>
        <span class="pill" class:selected={tab === id}>
          <Icon name={icon} outline={tab !== id} />
          {#if id === 'jobs' && active.length}<span class="badge label-m" aria-label="{active.length} en cours">{active.length}</span>{/if}
        </span>
        <span class="label label-m" class:em={tab === id}>{label}</span>
      </button>
    {/each}
  </nav>

  <main>
    <div class="page">
      {#if store.health && !store.health.ffmpeg}
        <p class="card error body-m">ffmpeg est introuvable : installez-le (<code>brew install ffmpeg</code>) pour encoder.</p>
      {:else if store.health && !store.health.videotoolbox}
        <p class="card error body-m">Cette version de ffmpeg ne supporte pas <code>hevc_videotoolbox</code>.</p>
      {/if}
      {#key tab}
        <div class="view">
          {#if tab === 'new'}
            <NewTask onsubmitted={() => (tab = 'jobs')} />
          {:else if tab === 'jobs'}
            <Jobs jobs={active} oncreate={() => (tab = 'new')} />
          {:else if tab === 'history'}
            <History />
          {:else}
            <Luts />
          {/if}
        </div>
      {/key}
    </div>
  </main>
</div>

<Snackbar />

<style>
  .shell { display: flex; min-height: 100vh; }
  main { flex: 1; min-width: 0; padding: 24px 16px 104px; }
  .page { max-width: 720px; margin: 0 auto; display: grid; gap: 16px; }
  .view { animation: enter var(--effects-slow); }
  @keyframes enter { from { opacity: 0; transform: translateY(8px); } }

  /* Compact: navigation bar at the bottom */
  .nav {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    height: 80px;
    display: flex;
    justify-content: space-around;
    align-items: center;
    background: var(--md-surface-container);
    z-index: 5;
  }
  .brand { display: none; }
  .nav-item {
    flex: 1;
    display: grid;
    justify-items: center;
    gap: 4px;
    padding: 12px 0 16px;
    background: none;
    color: var(--md-on-surface-variant);
  }
  .nav-item[aria-current='page'] { color: var(--md-on-surface); }
  .pill {
    position: relative;
    width: 56px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: var(--shape-full);
    color: var(--md-on-surface-variant);
    transition: background-color var(--effects-default), color var(--effects-default);
  }
  .pill.selected { background: var(--md-secondary-container); color: var(--md-on-secondary-container); }
  .pill :global(svg) { width: 24px; height: 24px; }
  .label.em { font-weight: 700; color: var(--md-secondary); }
  .badge {
    position: absolute;
    top: 0;
    right: 6px;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: var(--shape-full);
    background: var(--md-error);
    color: var(--md-on-error);
    font-size: 0.6875rem;
    line-height: 16px;
    text-align: center;
  }
  .status { width: 8px; height: 8px; border-radius: 50%; background: var(--md-primary); }
  .status.off { background: var(--md-error); }

  /* Medium and above (>= 600dp): collapsed navigation rail, 96dp wide */
  @media (min-width: 600px) {
    .shell { padding-left: 96px; }
    main { padding: 32px 24px; }
    .nav {
      top: 0;
      right: auto;
      bottom: 0;
      width: 96px;
      height: auto;
      flex-direction: column;
      justify-content: flex-start;
      gap: 12px;
      padding-top: 44px;
    }
    .nav-item { flex: none; width: 100%; padding: 4px 0; }
    .brand { display: grid; position: absolute; top: 16px; left: 0; right: 0; justify-items: center; }
  }
  @media (prefers-reduced-motion: reduce) { .view { animation: none; } }
</style>
