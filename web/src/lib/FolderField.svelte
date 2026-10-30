<script>
  import { api } from './api.js';
  import Icon from './Icon.svelte';
  import Dialog from './Dialog.svelte';
  import { notify } from './store.svelte.js';

  let { label, value = $bindable(''), placeholder = '', ext = '' } = $props();
  let open = $state(false);
  let browse = $state(null);
  let error = $state('');

  async function load(path) {
    try {
      browse = await api.get(`/api/fs/browse?path=${encodeURIComponent(path ?? '')}${ext ? `&ext=${ext}` : ''}`);
      error = '';
    } catch (e) {
      error = e.message;
    }
  }

  function openBrowser() {
    open = true;
    load(ext ? value.replace(/\/[^/]*$/, '') : value);
  }

  function choose() {
    value = browse.path;
    open = false;
  }

  async function pickNative() {
    try {
      const { path } = await api.post(`/api/fs/pick${ext ? '?file=true' : ''}`);
      if (path) value = path;
    } catch (e) {
      notify(e.message);
    }
  }
</script>

<div class="stack">
  <label class="tf trailing" class:filled={value}>
    <input bind:value {placeholder} spellcheck="false" />
    <span class="body-m">{label}</span>
    <button type="button" class="icon-btn trail" aria-label={ext ? 'Parcourir les fichiers' : 'Parcourir les dossiers'} onclick={openBrowser}>
      <Icon name={ext ? 'tune' : 'folder_open'} />
    </button>
  </label>
  <div class="row"><button type="button" class="btn text" onclick={pickNative}>{ext ? 'Choisir un fichier avec le Finder' : 'Choisir avec le Finder'}</button></div>
</div>

<Dialog bind:open title={ext ? 'Choisir un fichier' : 'Choisir un dossier'}>
  {#if browse}
    <div class="stack loose">
      <div class="row wrap">
        {#each browse.shortcuts as s}
          <button type="button" class="chip clickable" onclick={() => load(s.path)}>{s.name}</button>
        {/each}
      </div>
      <div class="row">
        <button type="button" class="icon-btn" aria-label="Dossier parent" disabled={!browse.parent} onclick={() => load(browse.parent)}>
          <Icon name="up" />
        </button>
        <label class="tf filled grow">
          <input value={browse.path} onkeydown={(e) => e.key === 'Enter' && load(e.currentTarget.value)} spellcheck="false" />
          <span class="body-m">Chemin</span>
        </label>
      </div>
      <ul class="list">
        {#each browse.dirs as d (d.path)}
          <li>
            <button type="button" class="list-item action" onclick={() => load(d.path)}>
              <span class="lead"><Icon name="folder" /></span>
              <span class="content body-l">{d.name}</span>
            </button>
          </li>
        {:else}
          {#if !browse.files.length}<li class="list-item"><span class="content body-m variant">Aucun sous-dossier</span></li>{/if}
        {/each}
        {#each browse.files as f (f.path)}
          <li>
            <button type="button" class="list-item action" onclick={() => { value = f.path; open = false; }}>
              <span class="lead"><Icon name="tune" /></span>
              <span class="content body-l">{f.name}</span>
            </button>
          </li>
        {/each}
      </ul>
      {#if !ext}<p class="body-s variant">{browse.video_count} vidéo(s) dans ce dossier</p>{/if}
    </div>
  {/if}
  {#if error}<p class="support err body-s">{error}</p>{/if}
  {#snippet actions()}
    <button type="button" class="btn text" onclick={() => (open = false)}>Annuler</button>
    {#if !ext}<button type="button" class="btn text" disabled={!browse} onclick={choose}>Choisir</button>{/if}
  {/snippet}
</Dialog>

<style>
  .grow { flex: 1; min-width: 0; }
  ul.list { max-height: 300px; overflow: auto; border-radius: var(--shape-m); }
  .chip.clickable { cursor: pointer; border: 0; }
  .chip.clickable:hover::before { opacity: 0.08; }
</style>
