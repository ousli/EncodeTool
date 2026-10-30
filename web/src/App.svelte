<script>
  import { onMount } from 'svelte';
  import { store, start } from './lib/store.svelte.js';
  import NewTask from './lib/NewTask.svelte';
  import Jobs from './lib/Jobs.svelte';
  import History from './lib/History.svelte';
  import Luts from './lib/Luts.svelte';

  let tab = $state('new');
  onMount(start);

  const active = $derived(store.jobs.filter((j) => !['done', 'failed', 'cancelled'].includes(j.status)));
  const tabs = $derived([
    ['new', 'Nouvelle tâche'],
    ['jobs', `Tâches${active.length ? ` (${active.length})` : ''}`],
    ['history', 'Historique'],
    ['luts', 'LUT'],
  ]);
</script>

<header>
  <h1>🎬 EncodeTool</h1>
  <nav>
    {#each tabs as [id, label]}
      <button class:selected={tab === id} onclick={() => (tab = id)}>{label}</button>
    {/each}
  </nav>
  <span class="spacer"></span>
  <span class="dot" class:on={store.connected} title={store.connected ? 'Connecté' : 'Déconnecté'}></span>
</header>

<main>
  {#if store.health && !store.health.ffmpeg}
    <p class="card error">ffmpeg est introuvable : installez-le (<code>brew install ffmpeg</code>) pour encoder.</p>
  {:else if store.health && !store.health.videotoolbox}
    <p class="card error">Cette version de ffmpeg ne supporte pas <code>hevc_videotoolbox</code>.</p>
  {/if}
  {#if tab === 'new'}
    <NewTask onsubmitted={() => (tab = 'jobs')} />
  {:else if tab === 'jobs'}
    <Jobs jobs={active} />
  {:else if tab === 'history'}
    <History />
  {:else}
    <Luts />
  {/if}
</main>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 12px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  h1 { margin: 0; font-size: 17px; }
  nav { display: flex; gap: 4px; }
  nav button { background: none; border-color: transparent; color: var(--muted); }
  nav button.selected { background: var(--panel-2); color: var(--text); border-color: var(--border); }
  .spacer { flex: 1; }
  .dot { width: 10px; height: 10px; border-radius: 50%; background: var(--err); }
  .dot.on { background: var(--ok); }
  main { max-width: 860px; margin: 24px auto; padding: 0 24px; display: grid; gap: 16px; }
</style>
