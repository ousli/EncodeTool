<script>
  import { api } from './api.js';
  import { store } from './store.svelte.js';
  import JobCard from './JobCard.svelte';

  const finished = $derived(store.jobs.filter((j) => ['done', 'failed', 'cancelled'].includes(j.status)).toReversed());
</script>

<div class="row">
  <h2>Historique</h2>
  <span class="spacer"></span>
  {#if finished.length}<button class="danger" onclick={() => api.del('/api/jobs')}>Tout effacer</button>{/if}
</div>
{#each finished as job (job.id)}
  <JobCard {job} />
{:else}
  <p class="card muted">Aucune tâche terminée.</p>
{/each}
