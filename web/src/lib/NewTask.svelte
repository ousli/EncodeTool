<script>
  import { api } from './api.js';
  import { store, KINDS, notify } from './store.svelte.js';
  import FolderField from './FolderField.svelte';
  import Icon from './Icon.svelte';

  let { onsubmitted } = $props();
  let kind = $state('encode');
  let source = $state('');
  let exportDir = $state('');
  let lutId = $state('');
  let busy = $state(false);

  const icons = { rename: 'edit', encode: 'movie', rename_encode: 'check', lut: 'tune' };
  const needsExport = $derived(kind !== 'rename');
  const descriptions = {
    rename: 'Ajoute la date de création (AAAA-MM-JJ_HHMM_) au début du nom des fichiers.',
    encode: 'Encode en H.265 10-bit (VideoToolbox) vers des fichiers .mov.',
    rename_encode: 'Encode en H.265 10-bit avec la date en préfixe du nom de sortie.',
    lut: 'Applique une LUT (LOG vers Rec.709) pendant l’encodage H.265 10-bit.',
  };

  async function submit(e) {
    e.preventDefault();
    busy = true;
    try {
      await api.post('/api/jobs', {
        kind,
        source,
        export: needsExport && exportDir ? exportDir : null,
        lut_id: kind === 'lut' ? Number(lutId) || null : null,
      });
      notify('Tâche ajoutée à la file');
      onsubmitted?.();
    } catch (err) {
      notify(err.message);
    } finally {
      busy = false;
    }
  }
</script>

<form class="stack loose" onsubmit={submit}>
  <h1 class="headline-s-em page-title">Nouvelle tâche</h1>

  <div class="stack">
    <div class="seg-group" role="radiogroup" aria-label="Type de tâche">
      {#each Object.entries(KINDS) as [id, label]}
        <button type="button" role="radio" aria-checked={kind === id} class="seg" class:selected={kind === id} onclick={() => (kind = id)}>
          {#if kind === id}<Icon name="check" />{:else}<Icon name={icons[id]} />{/if}
          {label}
        </button>
      {/each}
    </div>
    <p class="body-m variant">{descriptions[kind]}</p>
  </div>

  <div class="card low stack loose">
    <FolderField label="Dossier source" bind:value={source} placeholder="/Users/…/Rushes" />
    {#if needsExport}
      <FolderField label="Dossier d'export" bind:value={exportDir} placeholder="vide = source/export" />
    {/if}
    {#if kind === 'lut'}
      <div>
        <label class="tf select filled">
          <select bind:value={lutId}>
            <option value="">Choisir…</option>
            {#each store.luts as lut}<option value={lut.id}>{lut.name}</option>{/each}
          </select>
          <span class="body-m">LUT</span>
        </label>
        {#if !store.luts.length}<p class="support body-s">Ajoutez d'abord une LUT dans la section « LUT ».</p>{/if}
      </div>
    {/if}
  </div>

  <div class="row">
    <span class="spacer"></span>
    <button class="btn filled icon-lead" disabled={busy || !source || (kind === 'lut' && !lutId)}>
      <Icon name="play" /> Lancer
    </button>
  </div>
</form>
