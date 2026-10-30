<script>
  import { api } from './api.js';
  import { store, refreshLuts, notify } from './store.svelte.js';
  import Icon from './Icon.svelte';
  import FolderField from './FolderField.svelte';

  const f = store.lutForm;

  async function add(e) {
    e.preventDefault();
    try {
      await api.post('/api/luts', { name: f.name, path: f.path });
      f.name = f.path = '';
      await refreshLuts();
      notify('LUT ajoutée');
    } catch (err) {
      notify(err.message);
    }
  }

  async function remove(id) {
    try {
      await api.del(`/api/luts/${id}`);
      await refreshLuts();
      notify('LUT supprimée');
    } catch (err) {
      notify(err.message);
    }
  }
</script>

<div class="stack loose">
  <h1 class="headline-s-em page-title">LUT</h1>

  <form class="card low stack loose" onsubmit={add}>
    <h2 class="title-m-em">Ajouter une LUT</h2>
    <label class="tf" class:filled={f.name}>
      <input bind:value={f.name} placeholder="ex. S-Log2" />
      <span class="body-m">Nom</span>
    </label>
    <FolderField label="Fichier .cube" bind:value={f.path} placeholder="/chemin/vers/fichier.cube" ext="cube" />
    <div class="row">
      <span class="spacer"></span>
      <button class="btn filled icon-lead" disabled={!f.name || !f.path}><Icon name="add" />Ajouter</button>
    </div>
  </form>

  <h2 class="title-m-em">LUT enregistrées</h2>
  {#if store.luts.length}
    <ul class="list">
      {#each store.luts as lut (lut.id)}
        <li class="list-item">
          <span class="lead"><Icon name="tune" /></span>
          <div class="content">
            <p class="body-l">{lut.name}</p>
            <p class="body-s variant mono">{lut.path}</p>
          </div>
          <button class="icon-btn" aria-label="Supprimer {lut.name}" onclick={() => remove(lut.id)}><Icon name="delete" /></button>
        </li>
      {/each}
    </ul>
  {:else}
    <div class="empty">
      <Icon name="tune" outline />
      <p class="body-m">Aucune LUT enregistrée.</p>
    </div>
  {/if}
</div>
