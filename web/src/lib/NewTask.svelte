<script>
  import { api } from './api.js';
  import { store, PICKER, notify } from './store.svelte.js';
  import FolderField from './FolderField.svelte';
  import Icon from './Icon.svelte';

  let { onsubmitted } = $props();
  const form = store.form;
  let busy = $state(false);

  const icons = { rename: 'edit', encode: 'movie' };
  const encoding = $derived(form.kind === 'encode');
  const descriptions = {
    rename: 'Ajoute la date de création (AAAA-MM-JJ_HHMM_) au début du nom des fichiers.',
    encode: 'Encode en H.265 10-bit (VideoToolbox) vers des fichiers .mov.',
  };

  // The backend keeps one kind per task: map the options onto it.
  const apiKind = $derived(!encoding ? 'rename' : form.rename ? 'rename_encode' : form.useLut ? 'lut' : 'encode');
  const ready = $derived(form.source && (!encoding || !form.useLut || form.lutId));

  async function submit(e) {
    e.preventDefault();
    busy = true;
    try {
      await api.post('/api/jobs', {
        kind: apiKind,
        source: form.source,
        export: encoding && form.exportDir ? form.exportDir : null,
        keep_metadata: form.keepMetadata,
        lut_id: encoding && form.useLut ? Number(form.lutId) || null : null,
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
      {#each Object.entries(PICKER) as [id, label]}
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
    {#if encoding}
      <FolderField label="Dossier d'export" bind:value={form.exportDir} placeholder="vide = source/export" />
    {/if}
  </div>

  {#if encoding}
    <div class="stack">
      <h2 class="title-m-em">Options</h2>

      <div class="option" class:on={form.useLut}>
        <label class="switch-row">
          <span class="lead"><Icon name="tune" /></span>
          <span class="stack grow">
            <span class="title-m">Appliquer une LUT</span>
            <span class="body-s variant">Convertit le LOG en Rec.709 pendant l'encodage.</span>
          </span>
          <input type="checkbox" role="switch" class="switch" bind:checked={form.useLut} />
        </label>
        {#if form.useLut}
          <div class="sub">
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

      <div class="option" class:on={form.rename}>
        <label class="switch-row">
          <span class="lead"><Icon name="edit" /></span>
          <span class="stack grow">
            <span class="title-m">Renommer avec la date</span>
            <span class="body-s variant">Préfixe le fichier exporté par AAAA-MM-JJ_HHMM_.</span>
          </span>
          <input type="checkbox" role="switch" class="switch" bind:checked={form.rename} />
        </label>
      </div>

      <div class="option">
        <label class="switch-row">
          <span class="lead"><Icon name="folder" /></span>
          <span class="stack grow">
            <span class="title-m">Conserver les métadonnées</span>
            <span class="body-s variant">EXIF, date, GPS, appareil… copiés dans la vidéo exportée.</span>
          </span>
          <input type="checkbox" role="switch" class="switch" bind:checked={form.keepMetadata} />
        </label>
      </div>
    </div>
  {/if}

  <div class="row">
    <span class="spacer"></span>
    <button class="btn filled icon-lead" disabled={busy || !ready}>
      <Icon name="play" /> Lancer
    </button>
  </div>
</form>

<style>
  .grow { flex: 1; min-width: 0; }
  .option {
    padding: 16px;
    border-radius: var(--shape-m);
    background: var(--md-surface-container-low);
    transition: background-color var(--effects-default);
  }
  .option.on { background: var(--md-primary-container); color: var(--md-on-primary-container); --tf-bg: var(--md-primary-container); }
  .option.on .variant { color: inherit; opacity: 0.8; }
  .option .lead { display: grid; flex: none; }
  .option .lead :global(svg) { width: 24px; height: 24px; }
  .switch-row { gap: 16px; }
  .sub { margin-top: 16px; }
</style>
