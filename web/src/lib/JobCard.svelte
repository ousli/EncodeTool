<script>
  import { api } from './api.js';
  import { KINDS, STATUS, formatEta, formatDate, formatDuration } from './store.svelte.js';

  let { job } = $props();
  let error = $state('');

  const finished = $derived(['done', 'failed', 'cancelled'].includes(job.status));
  const live = $derived(['running', 'paused'].includes(job.status));

  async function act(action) {
    try {
      error = '';
      if (action === 'remove') await api.del(`/api/jobs/${job.id}`);
      else await api.post(`/api/jobs/${job.id}/${action}`);
    } catch (e) {
      error = e.message;
    }
  }
</script>

<div class="card job">
  <div class="row">
    <strong>{KINDS[job.kind]}{job.lut_name ? ` · ${job.lut_name}` : ''}</strong>
    <span class="badge {job.status}">{STATUS[job.status]}</span>
    <span class="spacer"></span>
    {#if job.status === 'running'}<button onclick={() => act('pause')}>Pause</button>{/if}
    {#if job.status === 'paused'}<button onclick={() => act('resume')}>Reprendre</button>{/if}
    {#if !finished}<button class="danger" onclick={() => act('cancel')}>Annuler</button>{/if}
    {#if finished}<button class="danger" onclick={() => act('remove')}>Supprimer</button>{/if}
  </div>
  <div class="muted mono path">{job.source}{job.export ? ` → ${job.export}` : ''}</div>

  {#if live}
    <div class="file">{job.current_file} <span class="muted">({job.index}/{job.total})</span></div>
    <div class="bar"><div style="width: {job.file_percent}%"></div></div>
    <div class="stats muted">
      <span>{Math.floor(job.file_percent)}%</span>
      <span>ETA {formatEta(job.eta_secs)}</span>
      <span>{job.fps != null ? `${job.fps} fps` : '-- fps'}</span>
      <span>{job.bitrate ?? '--'}</span>
      {#if job.speed}<span>{job.speed}×</span>{/if}
    </div>
    <div class="bar total"><div style="width: {job.percent}%"></div></div>
    <div class="stats muted"><span>Total {Math.floor(job.percent)}%</span></div>
  {:else if finished}
    <div class="stats muted">
      <span>{job.done_files}/{job.total} fichier(s)</span>
      <span>{formatDate(job.finished_at)}</span>
      <span>{formatDuration(job)}</span>
    </div>
  {:else}
    <div class="muted">En attente de son tour…</div>
  {/if}
  {#if job.error}<pre class="error">{job.error}</pre>{/if}
  {#if error}<p class="error">{error}</p>{/if}
</div>

<style>
  .job { display: grid; gap: 8px; }
  .path { font-size: 12px; overflow-wrap: anywhere; }
  .badge { font-size: 12px; padding: 2px 8px; border-radius: 999px; background: var(--panel-2); }
  .badge.running { color: var(--accent); }
  .badge.paused { color: var(--warn); }
  .badge.done { color: var(--ok); }
  .badge.failed { color: var(--err); }
  .bar { height: 10px; background: var(--bg); border-radius: 999px; overflow: hidden; }
  .bar div { height: 100%; background: var(--accent); transition: width 0.4s linear; }
  .bar.total div { background: var(--ok); }
  .stats { display: flex; gap: 16px; flex-wrap: wrap; font-size: 13px; }
  .file { overflow-wrap: anywhere; }
  pre { margin: 0; white-space: pre-wrap; font-size: 12px; }
</style>
