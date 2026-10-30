<script>
  import { api } from './api.js';
  import { store, refreshLuts } from './store.svelte.js';

  let name = $state('');
  let path = $state('');
  let error = $state('');

  async function add(e) {
    e.preventDefault();
    try {
      error = '';
      await api.post('/api/luts', { name, path });
      name = path = '';
      await refreshLuts();
    } catch (err) {
      error = err.message;
    }
  }

  async function remove(id) {
    await api.del(`/api/luts/${id}`);
    await refreshLuts();
  }
</script>

<form class="card" onsubmit={add}>
  <h2>Ajouter une LUT</h2>
  <div class="row">
    <input bind:value={name} placeholder="Nom (ex. S-Log2)" />
    <input bind:value={path} placeholder="/chemin/vers/fichier.cube" spellcheck="false" />
    <button class="primary" disabled={!name || !path}>Ajouter</button>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
</form>

<div class="card">
  <h2>LUT enregistrées</h2>
  {#each store.luts as lut (lut.id)}
    <div class="row item">
      <strong>{lut.name}</strong>
      <span class="muted mono path">{lut.path}</span>
      <button class="danger" onclick={() => remove(lut.id)}>Supprimer</button>
    </div>
  {:else}
    <p class="muted">Aucune LUT enregistrée.</p>
  {/each}
</div>

<style>
  form { display: grid; gap: 10px; }
  .item { padding: 8px 0; border-top: 1px solid var(--border); }
  .path { flex: 1; font-size: 12px; overflow-wrap: anywhere; }
</style>
