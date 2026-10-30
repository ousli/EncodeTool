<script>
  import { api } from './api.js';
  import { store, notify } from './store.svelte.js';
  import JobCard from './JobCard.svelte';
  import Dialog from './Dialog.svelte';
  import Icon from './Icon.svelte';

  const finished = $derived(store.jobs.filter((j) => ['done', 'failed', 'cancelled'].includes(j.status)).toReversed());
  let confirm = $state(false);

  async function clearAll() {
    confirm = false;
    try {
      await api.del('/api/jobs');
      notify('Historique effacé');
    } catch (e) {
      notify(e.message);
    }
  }
</script>

<div class="stack loose">
  <div class="row">
    <h1 class="headline-s-em">Historique</h1>
    <span class="spacer"></span>
    {#if finished.length}<button class="btn text danger" onclick={() => (confirm = true)}>Tout effacer</button>{/if}
  </div>
  {#each finished as job (job.id)}
    <JobCard {job} />
  {:else}
    <div class="empty">
      <Icon name="history" outline />
      <p class="title-m">Aucune tâche terminée</p>
      <p class="body-m">Les tâches terminées, échouées ou annulées apparaissent ici.</p>
    </div>
  {/each}
</div>

<Dialog bind:open={confirm} title="Effacer l'historique ?">
  Les {finished.length} tâche(s) terminée(s) seront supprimées de la liste. Les fichiers générés ne sont pas touchés.
  {#snippet actions()}
    <button class="btn text" onclick={() => (confirm = false)}>Annuler</button>
    <button class="btn text danger" onclick={clearAll}>Effacer</button>
  {/snippet}
</Dialog>
