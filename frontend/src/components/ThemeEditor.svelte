<script lang="ts">
  import { t } from '../lib/i18n';
  import type { AppSettings } from '../lib/types/index';

  let {
    settings,
    uiLang = 'en',
    onUpdateSetting,
  }: {
    settings: AppSettings | null;
    uiLang?: string;
    onUpdateSetting: (key: string, value: unknown) => void;
  } = $props();

  interface Group {
    id: string;
    label: string;
    variables: string[];
  }

  const GROUPS: Group[] = [
    {
      id: 'surfaces',
      label: 'Surfaces',
      variables: [
        '--color-app-background',
        '--color-surface-primary',
        '--color-surface-raised',
        '--color-surface-hover',
        '--color-surface-active',
        '--color-overlay',
        '--color-modal-surface',
        '--color-tooltip-surface',
        '--color-scrollbar',
        '--color-scrollbar-hover',
      ],
    },
    {
      id: 'text',
      label: 'Text',
      variables: [
        '--color-text-primary',
        '--color-text-secondary',
        '--color-text-muted',
        '--color-text-disabled',
        '--color-selection',
      ],
    },
    {
      id: 'typing',
      label: 'Typing',
      variables: [
        '--color-typing-pending',
        '--color-typing-current',
        '--color-typing-correct',
        '--color-typing-incorrect',
        '--color-typing-corrected',
        '--color-caret',
      ],
    },
    {
      id: 'keyboard',
      label: 'Keyboard',
      variables: [
        '--color-key-background',
        '--color-key-border',
        '--color-key-active',
        '--color-key-pressed',
      ],
    },
    {
      id: 'charts',
      label: 'Charts',
      variables: [
        '--color-chart-primary',
        '--color-chart-secondary',
        '--color-chart-positive',
        '--color-chart-negative',
        '--color-chart-grid',
        '--color-chart-axis',
        '--color-chart-label',
        '--color-chart-tooltip-background',
        '--color-chart-tooltip-border',
        '--color-chart-selected',
      ],
    },
    {
      id: 'misc',
      label: 'Misc',
      variables: [
        '--color-border',
        '--color-border-strong',
        '--color-accent',
        '--color-accent-hover',
        '--color-accent-active',
        '--color-accent-text',
        '--color-focus-ring',
        '--color-success',
        '--color-warning',
        '--color-error',
        '--color-information',
        '--color-progress-track',
        '--color-progress-fill',
      ],
    },
  ];

  function parseStored(): Record<string, string> {
    const raw = settings?.custom_theme_colors ?? '';
    if (!raw) return {};
    try {
      const parsed = JSON.parse(raw) as Record<string, unknown>;
      const out: Record<string, string> = {};
      for (const [key, value] of Object.entries(parsed)) {
        if (typeof value === 'string') out[key] = value;
      }
      return out;
    } catch {
      return {};
    }
  }

  let colors = $state<Record<string, string>>(parseStored());
  let status = $state('');

  // Синхронизация с сохранённой темой при смене settings (например, после
  // переключения темы в другом месте).
  $effect(() => {
    const stored = parseStored();
    const storedKeys = Object.keys(stored);
    const currentKeys = Object.keys(colors);
    if (
      storedKeys.length !== currentKeys.length
      || storedKeys.some((key) => stored[key] !== colors[key])
    ) {
      colors = stored;
    }
  });

  function setColor(variable: string, value: string) {
    colors = { ...colors, [variable]: value };
  }

  function resetColor(variable: string) {
    const next = { ...colors };
    delete next[variable];
    colors = next;
  }

  async function save() {
    try {
      await onUpdateSetting('custom_theme_colors', JSON.stringify(colors));
      status = t(uiLang, 'theme_editor.saved');
    } catch {
      status = t(uiLang, 'theme_editor.save_failed');
    }
  }

  function resetAll() {
    colors = {};
    status = '';
  }

  function randomize() {
    // Случайный базовый оттенок; производные цвета строятся от него так,
    // чтобы палитра оставалась связной (тёмная тема).
    const hue = Math.floor(Math.random() * 360);
    const hsl = (h: number, s: number, l: number) => {
      const sNorm = s / 100;
      const lNorm = l / 100;
      const a = sNorm * Math.min(lNorm, 1 - lNorm);
      const f = (n: number) => {
        const k = (n + h / 30) % 12;
        const c = lNorm - a * Math.max(-1, Math.min(k - 3, Math.min(9 - k, 1)));
        return Math.round(255 * c).toString(16).padStart(2, '0');
      };
      return `#${f(0)}${f(8)}${f(4)}`;
    };
    const next: Record<string, string> = {
      '--color-app-background': hsl(hue, 18, 7),
      '--color-surface-primary': hsl(hue, 16, 10),
      '--color-surface-raised': hsl(hue, 15, 13),
      '--color-surface-hover': hsl(hue, 14, 16),
      '--color-surface-active': hsl(hue, 14, 20),
      '--color-text-primary': hsl(hue, 20, 90),
      '--color-text-secondary': hsl(hue, 12, 68),
      '--color-text-muted': hsl(hue, 10, 52),
      '--color-text-disabled': hsl(hue, 8, 42),
      '--color-border': hsl(hue, 12, 26),
      '--color-border-strong': hsl(hue, 12, 36),
      '--color-accent': hsl(hue, 45, 72),
      '--color-accent-hover': hsl(hue, 50, 82),
      '--color-accent-active': hsl(hue, 55, 90),
      '--color-accent-text': hsl(hue, 18, 8),
      '--color-focus-ring': hsl(hue, 50, 85),
      '--color-selection': hsl(hue, 20, 30),
      '--color-caret': hsl(hue, 55, 88),
      '--color-typing-pending': hsl(hue, 10, 62),
      '--color-typing-current': hsl(hue, 25, 95),
      '--color-typing-correct': hsl(hue, 20, 70),
      '--color-typing-incorrect': hsl(0, 55, 72),
      '--color-typing-corrected': hsl(35, 55, 68),
      '--color-key-background': hsl(hue, 15, 12),
      '--color-key-border': hsl(hue, 12, 28),
      '--color-key-active': hsl(hue, 30, 80),
      '--color-key-pressed': hsl(hue, 40, 90),
      '--color-success': hsl(140, 30, 68),
      '--color-warning': hsl(40, 45, 68),
      '--color-error': hsl(0, 45, 70),
      '--color-information': hsl(hue, 30, 75),
      '--color-chart-primary': hsl(hue, 25, 82),
      '--color-chart-secondary': hsl(hue, 12, 60),
      '--color-chart-positive': hsl(140, 30, 68),
      '--color-chart-negative': hsl(0, 45, 70),
      '--color-chart-grid': hsl(hue, 12, 28),
      '--color-chart-axis': hsl(hue, 10, 48),
      '--color-chart-label': hsl(hue, 15, 70),
      '--color-chart-tooltip-background': hsl(hue, 15, 14),
      '--color-chart-tooltip-border': hsl(hue, 12, 36),
      '--color-chart-selected': hsl(hue, 30, 95),
      '--color-progress-track': hsl(hue, 14, 20),
      '--color-progress-fill': hsl(hue, 45, 72),
      '--color-overlay': hsl(hue, 18, 8),
      '--color-modal-surface': hsl(hue, 15, 13),
      '--color-tooltip-surface': hsl(hue, 15, 13),
      '--color-scrollbar': hsl(hue, 12, 26),
      '--color-scrollbar-hover': hsl(hue, 45, 72),
    };
    colors = next;
    status = '';
  }

  function isCustomActive(): boolean {
    return settings?.theme === 'custom';
  }
</script>

<div class="theme-editor">
  <div class="editor-header">
    <h3>{t(uiLang, 'theme_editor.title')}</h3>
    <div class="editor-actions">
      <button class="editor-btn" onclick={randomize}>{t(uiLang, 'theme_editor.randomize')}</button>
      <button class="editor-btn" onclick={resetAll}>{t(uiLang, 'theme_editor.reset')}</button>
      <button class="editor-btn primary" onclick={save}>{t(uiLang, 'theme_editor.save')}</button>
    </div>
  </div>
  {#if status}<p class="editor-status">{status}</p>{/if}
  {#if !isCustomActive()}
    <p class="editor-hint">{t(uiLang, 'theme_editor.hint')}</p>
  {/if}

  {#each GROUPS as group}
    <section class="editor-group">
      <h4>{group.label}</h4>
      <div class="color-grid">
        {#each group.variables as variable}
          <label class="color-row" title={variable}>
            <input
              type="color"
              value={colors[variable] ?? '#888888'}
              oninput={(e) => setColor(variable, e.currentTarget.value)}
            />
            <span class="color-name">{variable.replace('--color-', '')}</span>
            <span class="color-hex">{colors[variable] ?? '—'}</span>
            {#if colors[variable]}
              <button
                class="color-reset"
                aria-label={t(uiLang, 'theme_editor.clear')}
                onclick={() => resetColor(variable)}
              >×</button>
            {/if}
          </label>
        {/each}
      </div>
    </section>
  {/each}
</div>

<style>
  .theme-editor { display: flex; flex-direction: column; gap: 0.75rem; }
  .editor-header { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 0.75rem; }
  .editor-header h3 { color: var(--main); font-size: 1.1rem; margin: 0; }
  .editor-actions { display: flex; gap: 0.5rem; }
  .editor-btn {
    background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main);
    padding: 0.35rem 0.9rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
  }
  .editor-btn.primary { background-color: var(--main); color: var(--color-accent-text); }
  .editor-btn:hover { opacity: 0.85; }
  .editor-status { color: var(--color-success); font-size: 0.75rem; margin: 0; }
  .editor-hint { color: var(--sub); font-size: 0.75rem; margin: 0; }
  .editor-group h4 {
    color: var(--sub); font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.06em;
    margin: 0.25rem 0 0.4rem;
  }
  .color-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 0.35rem; }
  .color-row {
    display: flex; align-items: center; gap: 0.5rem;
    background: var(--bg-sub); border: 1px solid var(--color-border);
    border-radius: 6px; padding: 0.3rem 0.5rem;
  }
  .color-row input[type='color'] {
    width: 28px; height: 22px; padding: 0; border: 1px solid var(--color-border-strong);
    border-radius: 4px; background: transparent; cursor: pointer;
  }
  .color-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--sub); font-size: 0.68rem; }
  .color-hex { color: var(--text); font-size: 0.68rem; font-variant-numeric: tabular-nums; }
  .color-reset {
    background: transparent; border: none; color: var(--sub); cursor: pointer;
    font-size: 0.8rem; line-height: 1; padding: 0 0.1rem;
  }
  .color-reset:hover { color: var(--error); }
</style>
