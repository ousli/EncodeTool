<script>
  import { api } from './api.js';
  import { store, KINDS, notify } from './store.svelte.js';
  import FolderField from './FolderField.svelte';
  import Icon from './Icon.svelte';

  let { onsubmitted } = $props();
  const form = store.form;
  let busy = $state(false);

  const icons = { rename: 'edit', encode: 'movie', rename_encode: 'check', lut: 'tune' };
  const needsExport = $derived(form.kind !== 'rename');
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
        kind: form.kind,
        source: form.source,
        export: needsExport && form.exportDir ? form.exportDir : null,
        keep_metadata: form.keepMetadata,
        lut_id: form.kind === 'lut' ? Number(form.lutId) || null : null,
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
        <button type="button" role="radio" aria-checked={form.kind === id} class="seg" class:selected={form.kind === id} onclick={() => (form.kind = id)}>
          {#if form.kind === id}<Icon name="check" />{:else}<Icon name={icons[id]} />{/if}
          {label}
        </button>
      {/each}
    </div>
    <p class="body-m variant">{descriptions[form.kind]}</p>
  </div>

  <div class="card low stack loose">
    <FolderField label="Dossier source" bind:value={form.source} placeholder="/Users/…/Rushes" />
    {#if needsExport}
      <FolderField label="Dossier d'export" bind:value={form.exportDir} placeholder="vide = source/export" />
    {/if}
    {#if needsExport}
      <label class="switch-row">
        <span class="stack">
          <span class="body-l">Conserver les métadonnées</span>
          <span class="body-s variant">EXIF, date, GPS, appareil… copiés dans la vidéo exportée.</span>
        </span>
        <input type="checkbox" role="switch" class="switch" bind:checked={form.keepMetadata} />
      </label>
    {/if}
    {#if form.kind === 'lut'}
      <div>
        <label class="tf select filled">
          <select bind:value={form.lutId}>
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
    <button class="btn filled icon-lead" disabled={busy || !form.source || (form.kind === 'lut' && !form.lutId)}>
      <Icon name="play" /> Lancer
    </button>
  </div>
</form>
