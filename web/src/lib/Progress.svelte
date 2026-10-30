<script>
  // Linear progress indicator: 4dp track, rounded ends, stop indicator.
  // Wavy (3dp amplitude, 40dp wavelength) while active; flat when paused/idle.
  let { value = 0, wavy = false, label = 'Progression' } = $props();
  const v = $derived(Math.max(0, Math.min(100, value)));
</script>

<div class="lp" class:wavy role="progressbar" aria-label={label} aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(v)}>
  <div class="track"></div>
  <div class="active" style="width: {v}%"></div>
  <div class="stop"></div>
</div>

<style>
  .lp { position: relative; height: 10px; width: 100%; }
  .track, .active {
    position: absolute;
    left: 0;
    top: 3px;
    height: 4px;
    border-radius: var(--shape-full);
  }
  .track { width: 100%; background: var(--md-secondary-container); }
  .active { background: var(--md-primary); min-width: 4px; transition: width var(--effects-slow); }
  .stop { position: absolute; right: 0; top: 3px; width: 4px; height: 4px; border-radius: 50%; background: var(--md-primary); }
  .wavy .active {
    top: 0;
    height: 10px;
    border-radius: 0;
    min-width: 0;
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='10' viewBox='0 0 40 10'%3E%3Cpath d='M0 5C6.7-1 13.3-1 20 5S33.3 11 40 5' fill='none' stroke='black' stroke-width='4' stroke-linecap='round'/%3E%3C/svg%3E") repeat-x 0 0 / 40px 10px;
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='10' viewBox='0 0 40 10'%3E%3Cpath d='M0 5C6.7-1 13.3-1 20 5S33.3 11 40 5' fill='none' stroke='black' stroke-width='4' stroke-linecap='round'/%3E%3C/svg%3E") repeat-x 0 0 / 40px 10px;
    animation: wave 1s linear infinite;
  }
  @keyframes wave {
    to { -webkit-mask-position: 40px 0; mask-position: 40px 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .wavy .active { animation: none; }
    .active { transition: none; }
  }
</style>
