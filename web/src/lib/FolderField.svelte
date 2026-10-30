<script>
  import { api } from './api.js';

  let { label, value = $bindable(''), placeholder = '' } = $props();
  let open = $state(false);
  let browse = $state(null);
  let error = $state('');

  async function load(path) {
    try {
      browse = await api.get(`/api/fs/browse?path=${encodeURIComponent(path ?? '')}`);
      error = '';
    } catch (e) {
      error = e.message;
    }
  }

  function openBrowser() {
    open = true;
    load(value);
  }

  function choose() {
    value = browse.path;
    open = false;
  }

  async function pickNative() {
    try {
      const { path } = await api.post('/api/fs/pick');
      if (path) value = path;
    } catch (e) {
      error = e.message;
    }
  }
</script>

<label>
  <span class="muted">{label}</span>
  <div class="row">
    <input bind:value {placeholder} spellcheck="false" />
    <button type="button" onclick={openBrowser}>Parcourir…</button>
    <button type="button" onclick={pickNative} title="Dialogue macOS">Finder…</button>
  </div>
</label>
{#if error && !open}<p class="error">{error}</p>{/if}

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => (open = false)}>
    <div class="modal card" onclick={(e) => e.stopPropagation()}>
      <h2>Choisir un dossier</h2>
      {#if browse}
        <div class="row shortcuts">
          {#each browse.shortcuts as s}
            <button type="button" onclick={() => load(s.path)}>{s.name}</button>
          {/each}
        </div>
        <div class="row path">
          <button type="button" disabled={!browse.parent} onclick={() => load(browse.parent)}>↑</button>
          <input value={browse.path} onkeydown={(e) => e.key === 'Enter' && load(e.currentTarget.value)} spellcheck="false" />
        </div>
        <ul>
          {#each browse.dirs as d}
            <li><button type="button" onclick={() => load(d.path)}>📁 {d.name}</button></li>
          {:else}
            <li class="muted">Aucun sous-dossier</li>
          {/each}
        </ul>
        <div class="row">
          <span class="muted">{browse.video_count} vidéo(s) dans ce dossier</span>
          <span class="spacer"></span>
          <button type="button" onclick={() => (open = false)}>Annuler</button>
          <button type="button" class="primary" onclick={choose}>Choisir ce dossier</button>
        </div>
      {/if}
      {#if error}<p class="error">{error}</p>{/if}
    </div>
  </div>
{/if}

<style>
  label { display: grid; gap: 4px; }
  .row input { flex: 1; }
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: grid;
    place-items: center;
    z-index: 10;
  }
  .modal { width: min(640px, 92vw); display: grid; gap: 10px; }
  .shortcuts { flex-wrap: wrap; }
  ul { list-style: none; margin: 0; padding: 0; max-height: 320px; overflow: auto; border: 1px solid var(--border); border-radius: 8px; }
  li button { width: 100%; text-align: left; border: 0; border-radius: 0; background: none; }
  li button:hover { background: var(--panel-2); }
  li.muted { padding: 10px; }
</style>
