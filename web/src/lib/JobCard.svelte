<script>
  import { api } from './api.js';
  import { KINDS, STATUS, formatEta, formatDate, formatDuration, notify } from './store.svelte.js';
  import Icon from './Icon.svelte';
  import Progress from './Progress.svelte';

  let { job } = $props();

  const finished = $derived(['done', 'failed', 'cancelled'].includes(job.status));
  const live = $derived(['running', 'paused'].includes(job.status));

  async function act(action) {
    try {
      if (action === 'remove') await api.del(`/api/jobs/${job.id}`);
      else await api.post(`/api/jobs/${job.id}/${action}`);
    } catch (e) {
      notify(e.message);
    }
  }
</script>

<article class="card stack loose" class:outlined={finished}>
  <div class="row wrap">
    <h3 class="title-m-em">{KINDS[job.kind]}{job.lut_name ? ` · ${job.lut_name}` : ''}</h3>
    <span class="chip {job.status}">{#if job.status === 'done'}<Icon name="check" />{/if}{STATUS[job.status]}</span>
  </div>
  <p class="body-s variant mono path">{job.source}{job.export ? ` → ${job.export}` : ''}</p>

  {#if live}
    <div class="stack">
      <p class="body-m file">{job.current_file} <span class="variant">({job.index}/{job.total})</span></p>
      <Progress value={job.file_percent} wavy={job.status === 'running'} label="Fichier en cours" />
      <div class="stats body-s variant">
        <span>{Math.floor(job.file_percent)} %</span>
        <span>ETA {formatEta(job.eta_secs)}</span>
        <span>{job.fps != null ? `${job.fps} fps` : '-- fps'}</span>
        <span>{job.bitrate ?? '--'}</span>
        {#if job.speed}<span>{job.speed}×</span>{/if}
      </div>
    </div>
    <div class="stack">
      <Progress value={job.percent} label="Progression totale" />
      <p class="body-s variant">Total {Math.floor(job.percent)} %</p>
    </div>
  {:else if finished}
    <div class="stats body-s variant">
      <span>{job.done_files}/{job.total} fichier(s)</span>
      <span>{formatDate(job.finished_at)}</span>
      <span>{formatDuration(job)}</span>
    </div>
  {:else}
    <p class="body-m variant">En attente de son tour…</p>
  {/if}
  {#if job.error}<pre class="body-s err">{job.error}</pre>{/if}

  <div class="row actions">
    <span class="spacer"></span>
    {#if job.status === 'running'}<button class="btn tonal icon-lead" onclick={() => act('pause')}><Icon name="pause" />Pause</button>{/if}
    {#if job.status === 'paused'}<button class="btn tonal icon-lead" onclick={() => act('resume')}><Icon name="play" />Reprendre</button>{/if}
    {#if !finished}<button class="btn text danger" onclick={() => act('cancel')}>Annuler</button>{/if}
    {#if finished}<button class="btn text" onclick={() => act('remove')}>Supprimer</button>{/if}
  </div>
</article>

<style>
  .path, .file { overflow-wrap: anywhere; }
  .stats { display: flex; gap: 16px; flex-wrap: wrap; }
  pre { margin: 0; white-space: pre-wrap; max-height: 160px; overflow: auto; padding: 12px; border-radius: var(--shape-s); background: var(--md-error-container); color: var(--md-on-error-container); font-family: var(--mono); }
</style>
