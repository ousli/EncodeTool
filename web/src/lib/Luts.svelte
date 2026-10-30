<script>
  import { api } from './api.js';
  import { store, refreshLuts, notify } from './store.svelte.js';
  import Icon from './Icon.svelte';

  let name = $state('');
  let path = $state('');

  async function add(e) {
    e.preventDefault();
    try {
      await api.post('/api/luts', { name, path });
      name = path = '';
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
    <label class="tf" class:filled={name}>
      <input bind:value={name} placeholder="ex. S-Log2" />
      <span class="body-m">Nom</span>
    </label>
    <label class="tf" class:filled={path}>
      <input bind:value={path} placeholder="/chemin/vers/fichier.cube" spellcheck="false" />
      <span class="body-m">Chemin du fichier .cube</span>
    </label>
    <div class="row">
      <span class="spacer"></span>
      <button class="btn filled icon-lead" disabled={!name || !path}><Icon name="add" />Ajouter</button>
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
