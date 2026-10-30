<script>
  import { api } from './api.js';
  import { KINDS, STATUS, formatEta, formatDate, formatDuration, notify } from './store.svelte.js';
  import Icon from './Icon.svelte';
  import Progress from './Progress.svelte';

  let { job } = $props();

  const finished = $derived(['done', 'failed', 'cancelled'].includes(job.status));
  const live = $derived(['running', 'paused'].includes(job.status));
  const graded = $derived(!!job.lut_name);
  const pct = $derived(Math.max(0, Math.min(100, job.file_percent)));
  let failed = $state(false);
  $effect(() => {
    job.thumb_v;
    failed = false;
  });

  async function act(action) {
    try {
      if (action === 'remove') await api.del(`/api/jobs/${job.id}`);
      else await api.post(`/api/jobs/${job.id}/${action}`);
    } catch (e) {
      notify(e.message);
    }
  }
</script>

<article class="card stack loose" class:outlined={finished} class:live>
  <div class="row wrap">
    <h3 class="title-m-em">{KINDS[job.kind]}{job.lut_name ? ` · ${job.lut_name}` : ''}</h3>
    <span class="chip {job.status}">{#if job.status === 'done'}<Icon name="check" />{/if}{STATUS[job.status]}</span>
  </div>

  {#if live}
    <div class="live-grid">
      <div class="preview" class:paused={job.status === 'paused'}>
        {#if job.thumb_v && !failed}
          <img src="/api/jobs/{job.id}/thumb/orig?v={job.thumb_v}" alt="Image d'origine de {job.current_file}" onerror={() => (failed = true)} />
          {#if graded}
            <img class="graded" style="clip-path: inset(0 {100 - pct}% 0 0)" src="/api/jobs/{job.id}/thumb/lut?v={job.thumb_v}" alt="" />
            <div class="wipe" style="left: {pct}%"><span></span></div>
            <span class="tag left label-m">Original</span>
            <span class="tag right label-m">{job.lut_name}</span>
          {/if}
        {:else}
          <div class="ph"><Icon name="movie" outline /></div>
        {/if}
        <span class="tag pct label-m">{Math.floor(pct)} %</span>
      </div>

      <div class="stack loose info">
        <div class="stack">
          <p class="title-m file">{job.current_file} <span class="variant body-m">({job.index}/{job.total})</span></p>
          <p class="body-s variant mono path">{job.source}{job.export ? ` → ${job.export}` : ''}</p>
        </div>
        <div class="stack">
          <Progress value={job.file_percent} wavy={job.status === 'running'} label="Fichier en cours" />
          <div class="stats body-s variant">
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
      </div>
    </div>
  {:else}
    <p class="body-s variant mono path">{job.source}{job.export ? ` → ${job.export}` : ''}</p>
    {#if finished}
      <div class="stats body-s variant">
        <span>{job.done_files}/{job.total} fichier(s)</span>
        <span>{formatDate(job.finished_at)}</span>
        <span>{formatDuration(job)}</span>
      </div>
    {:else}
      <p class="body-m variant">En attente de son tour…</p>
    {/if}
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

  .live-grid { display: grid; gap: 16px; }
  @media (min-width: 600px) { .live-grid { grid-template-columns: minmax(0, 5fr) minmax(0, 6fr); align-items: center; } }
  .info { min-width: 0; }

  .preview {
    position: relative;
    aspect-ratio: 16 / 9;
    border-radius: var(--shape-m);
    overflow: hidden;
    background: var(--md-surface-container-lowest);
    isolation: isolate;
  }
  .preview img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
  .preview.paused img { filter: saturate(0.5); }
  .graded { transition: clip-path var(--effects-slow); }
  .wipe {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 0;
    transition: left var(--effects-slow);
  }
  .wipe::before { content: ''; position: absolute; top: 0; bottom: 0; left: -1px; width: 2px; background: var(--md-surface); box-shadow: 0 0 4px rgb(0 0 0 / 0.5); }
  .wipe span {
    position: absolute;
    top: 50%;
    left: -12px;
    width: 24px;
    height: 24px;
    margin-top: -12px;
    border-radius: 50%;
    background: var(--md-primary);
    box-shadow: 0 1px 3px rgb(0 0 0 / 0.4);
  }
  .tag {
    position: absolute;
    padding: 2px 8px;
    border-radius: var(--shape-xs);
    background: rgb(0 0 0 / 0.55);
    color: #fff;
  }
  .tag.left { left: 8px; bottom: 8px; }
  .tag.right { right: 8px; bottom: 8px; max-width: 45%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tag.pct { left: 8px; top: 8px; }
  .ph { position: absolute; inset: 0; display: grid; place-items: center; color: var(--md-on-surface-variant); animation: pulse 1.6s ease-in-out infinite; }
  .ph :global(svg) { width: 48px; height: 48px; opacity: 0.5; }
  @keyframes pulse { 50% { opacity: 0.5; } }
  @media (prefers-reduced-motion: reduce) { .ph { animation: none; } .graded, .wipe { transition: none; } }
</style>
