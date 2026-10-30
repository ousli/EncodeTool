// Light/dark mode (follows the system unless chosen) and accent colour.
// Accent palettes come from the M3 HCT Tonal Spot scheme, computed by /m3-palette.js (loaded lazily).
const MODE_KEY = 'encodetool-theme';
const ACCENT_KEY = 'encodetool-accent';

export const ACCENTS = [
  { id: 'violet', name: 'Violet', seed: '#6750A4' },
  { id: 'bleu', name: 'Bleu', seed: '#2F6BDB' },
  { id: 'turquoise', name: 'Turquoise', seed: '#00897B' },
  { id: 'vert', name: 'Vert', seed: '#2E7D32' },
  { id: 'ambre', name: 'Ambre', seed: '#F9A825' },
  { id: 'orange', name: 'Orange', seed: '#E65100' },
  { id: 'rose', name: 'Rose', seed: '#C2185B' },
  { id: 'rouge', name: 'Rouge', seed: '#B3261E' },
];

const TOKENS = {
  primary: 'primary',
  'on-primary': 'on-primary',
  'primary-container': 'primary-container',
  'on-primary-container': 'on-primary-container',
  'secondary-container': 'secondary-container',
  'on-secondary-container': 'on-secondary-container',
  surface: 'surface',
  'sc-lowest': 'surface-container-lowest',
  'sc-low': 'surface-container-low',
  sc: 'surface-container',
  'sc-high': 'surface-container-high',
  'sc-highest': 'surface-container-highest',
  outline: 'outline',
  'outline-variant': 'outline-variant',
  'on-surface-variant': 'on-surface-variant',
};

const read = (key) => {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
};
const write = (key, value) => {
  try {
    localStorage.setItem(key, value);
  } catch {}
};

const root = document.documentElement;
const mq = window.matchMedia('(prefers-color-scheme: dark)');

export const theme = $state({
  mode: ['dark', 'light'].includes(read(MODE_KEY)) ? read(MODE_KEY) : null,
  accent: ACCENTS.some((a) => a.id === read(ACCENT_KEY)) ? read(ACCENT_KEY) : 'violet',
  systemDark: mq.matches,
});

export const isDark = () => (theme.mode ? theme.mode === 'dark' : theme.systemDark);

let palette;
function loadPalette() {
  palette ??= new Promise((resolve, reject) => {
    if (window.M3Palette) return resolve(window.M3Palette);
    const s = document.createElement('script');
    s.src = '/m3-palette.js';
    s.onload = () => resolve(window.M3Palette);
    s.onerror = reject;
    document.head.appendChild(s);
  });
  return palette;
}

async function apply() {
  if (theme.mode) root.dataset.theme = theme.mode;
  else delete root.dataset.theme;
  const dark = isDark();
  const accent = ACCENTS.find((a) => a.id === theme.accent);
  for (const css of Object.values(TOKENS)) root.style.removeProperty(`--md-${css}`);
  if (accent.id !== 'violet') {
    try {
      const colors = (await loadPalette())(accent.seed, dark);
      // Ignore a stale result if the choice changed while the script was loading
      if (theme.accent !== accent.id || isDark() !== dark) return;
      for (const [key, css] of Object.entries(TOKENS)) root.style.setProperty(`--md-${css}`, colors[key]);
    } catch {}
  }
  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) meta.content = getComputedStyle(root).getPropertyValue('--md-surface').trim();
}

export function toggleMode() {
  const next = isDark() ? 'light' : 'dark';
  const change = () => {
    theme.mode = next;
    write(MODE_KEY, next);
    apply();
  };
  const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (!document.startViewTransition || reduce) return change();
  document.startViewTransition(change);
}

export function setAccent(id) {
  theme.accent = id;
  write(ACCENT_KEY, id);
  apply();
}

mq.addEventListener('change', (e) => {
  theme.systemDark = e.matches;
  apply();
});
apply();
