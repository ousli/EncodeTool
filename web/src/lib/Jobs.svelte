<script>
  import JobCard from './JobCard.svelte';

  let { jobs } = $props();
</script>

{#each jobs as job (job.id)}
  <JobCard {job} />
{:else}
  <p class="card muted">Aucune tâche en cours. Lancez-en une depuis « Nouvelle tâche ».</p>
{/each}
