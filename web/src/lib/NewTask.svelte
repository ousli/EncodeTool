<script>
  import { api } from './api.js';
  import { store, KINDS } from './store.svelte.js';
  import FolderField from './FolderField.svelte';

  let { onsubmitted } = $props();
  let kind = $state('encode');
  let source = $state('');
  let exportDir = $state('');
  let lutId = $state('');
  let error = $state('');
  let busy = $state(false);

  const needsExport = $derived(kind !== 'rename');
  const descriptions = {
    rename: 'Ajoute la date de création (AAAA-MM-JJ_HHMM_) au début du nom des fichiers.',
    encode: 'Encode en H.265 10-bit (VideoToolbox) vers des fichiers .mov.',
    rename_encode: 'Encode en H.265 10-bit avec la date en préfixe du nom de sortie.',
    lut: 'Applique une LUT (LOG vers Rec.709) pendant l’encodage H.265 10-bit.',
  };

  async function submit(e) {
    e.preventDefault();
    error = '';
    busy = true;
    try {
      await api.post('/api/jobs', {
        kind,
        source,
        export: needsExport && exportDir ? exportDir : null,
        lut_id: kind === 'lut' ? Number(lutId) || null : null,
      });
      onsubmitted?.();
    } catch (err) {
      error = err.message;
    } finally {
      busy = false;
    }
  }
</script>

<form class="card" onsubmit={submit}>
  <h2>Nouvelle tâche</h2>
  <div class="kinds">
    {#each Object.entries(KINDS) as [id, label]}
      <label class:selected={kind === id}>
        <input type="radio" bind:group={kind} value={id} />
        {label}
      </label>
    {/each}
  </div>
  <p class="muted">{descriptions[kind]}</p>

  <FolderField label="Dossier source" bind:value={source} placeholder="/Users/…/Rushes" />
  {#if needsExport}
    <FolderField label="Dossier d'export (vide = source/export)" bind:value={exportDir} placeholder="source/export" />
  {/if}
  {#if kind === 'lut'}
    <label class="field">
      <span class="muted">LUT</span>
      <select bind:value={lutId}>
        <option value="">Choisir…</option>
        {#each store.luts as lut}
          <option value={lut.id}>{lut.name}</option>
        {/each}
      </select>
      {#if !store.luts.length}<span class="muted">Ajoutez d'abord une LUT dans l'onglet « LUT ».</span>{/if}
    </label>
  {/if}

  {#if error}<p class="error">{error}</p>{/if}
  <div class="row">
    <span class="spacer"></span>
    <button class="primary" disabled={busy || !source}>Lancer</button>
  </div>
</form>

<style>
  form { display: grid; gap: 14px; }
  p { margin: 0; }
  .kinds { display: flex; flex-wrap: wrap; gap: 8px; }
  .kinds label {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
  }
  .kinds label.selected { border-color: var(--accent); background: var(--panel-2); }
  .kinds input { width: auto; }
  .field { display: grid; gap: 4px; }
</style>
