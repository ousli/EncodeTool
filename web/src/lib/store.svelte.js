import { api } from './api.js';

export const store = $state({
  jobs: [],
  luts: [],
  health: null,
  connected: false,
  snack: null,
  form: { kind: 'encode', source: '', exportDir: '', lutId: '' },
  lutForm: { name: '', path: '' },
});

let snackId = 0;
export function notify(text) {
  store.snack = { id: ++snackId, text };
}

export async function refreshLuts() {
  store.luts = await api.get('/api/luts');
}

export function start() {
  api.get('/api/health').then((h) => (store.health = h)).catch(() => {});
  refreshLuts().catch(() => {});
  const source = new EventSource('/api/events');
  source.onopen = () => (store.connected = true);
  source.onerror = () => (store.connected = false);
  source.onmessage = (e) => (store.jobs = JSON.parse(e.data));
  return () => source.close();
}

export const KINDS = {
  rename: 'Renommer',
  encode: 'Réencoder',
  rename_encode: 'Renommer + réencoder',
  lut: 'Appliquer une LUT',
};

export const STATUS = {
  queued: 'En attente',
  running: 'En cours',
  paused: 'En pause',
  done: 'Terminée',
  failed: 'Échec',
  cancelled: 'Annulée',
};

export function formatEta(secs) {
  if (secs == null) return '--';
  const p = (n) => String(n).padStart(2, '0');
  return `${p(Math.floor(secs / 3600))}:${p(Math.floor(secs / 60) % 60)}:${p(secs % 60)}`;
}

export function formatDate(ts) {
  return ts ? new Date(ts * 1000).toLocaleString('fr-FR') : '';
}

export function formatDuration(job) {
  if (!job.started_at || !job.finished_at) return '';
  const s = job.finished_at - job.started_at;
  return s < 60 ? `${s} s` : `${Math.floor(s / 60)} min ${s % 60} s`;
}
