<script>
  import JobCard from './JobCard.svelte';
  import Icon from './Icon.svelte';

  let { jobs, oncreate } = $props();
</script>

<div class="stack loose">
  <h1 class="headline-s-em page-title">Tâches</h1>
  {#each jobs as job (job.id)}
    <JobCard {job} />
  {:else}
    <div class="empty">
      <Icon name="movie" outline />
      <p class="title-m">Aucune tâche en cours</p>
      <p class="body-m">Lancez un encodage ou un renommage pour suivre sa progression ici.</p>
      <button class="btn tonal icon-lead" onclick={oncreate}><Icon name="add" />Nouvelle tâche</button>
    </div>
  {/each}
</div>
