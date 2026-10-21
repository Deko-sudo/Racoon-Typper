// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Feature store for settings and themes: loads settings/themes, applies the
// active theme (including the frontend-synthesized custom theme), and owns
// the inline CSS-variable bookkeeping on :root.

import * as ipc from '../api/ipc';
import type { AppSettings, ThemeInfo } from '../types/index';

export interface SettingsStoreDeps {
  setError: (message: string) => void;
}

export function createSettingsStore(deps: SettingsStoreDeps) {
  let settings = $state<AppSettings | null>(null);
  let uiLang = $state('en');
  let themes = $state<ThemeInfo[]>([]);
  let activeTheme = $state('racoon_graphite');
  const appliedThemeVariables = new Set<string>();

  let mainFontSize = $derived(`${settings?.font_size ?? 24}px`);

  async function loadThemes() {
    themes = await ipc.getThemes();
  }

  async function loadSettings() {
    settings = await ipc.getSettings();
    activeTheme = settings.theme;
    uiLang = settings.ui_language || 'en';
    await applyTheme(activeTheme);
  }

  /// Синтезирует CSS кастомной темы из JSON-объекта { "--color-*": "#rrggbb" }.
  /// Недостающие переменные наследуются от дефолтной темы (racoon_graphite),
  /// legacy-алиасы (--bg/--main/…) выводятся из семантических токенов.
  function buildCustomThemeCss(json: string): string {
    let colors: Record<string, string> = {};
    try {
      const parsed = JSON.parse(json) as Record<string, unknown>;
      for (const [key, value] of Object.entries(parsed)) {
        if (typeof value === 'string' && key.startsWith('--')) colors[key] = value;
      }
    } catch {
      colors = {};
    }
    const fallback = (key: string, defaultHex: string) => colors[key] ?? defaultHex;
    const lines: string[] = [];
    lines.push(':root {');
    lines.push(`  --color-app-background: ${fallback('--color-app-background', '#0d0f12')};`);
    lines.push(`  --color-surface-primary: ${fallback('--color-surface-primary', '#15181d')};`);
    lines.push(`  --color-surface-raised: ${fallback('--color-surface-raised', '#1c2027')};`);
    lines.push(`  --color-surface-hover: ${fallback('--color-surface-hover', '#252a32')};`);
    lines.push(`  --color-surface-active: ${fallback('--color-surface-active', '#2e343e')};`);
    lines.push(`  --color-text-primary: ${fallback('--color-text-primary', '#e7e9ed')};`);
    lines.push(`  --color-text-secondary: ${fallback('--color-text-secondary', '#adb3bd')};`);
    lines.push(`  --color-text-muted: ${fallback('--color-text-muted', '#8a919c')};`);
    lines.push(`  --color-text-disabled: ${fallback('--color-text-disabled', '#707885')};`);
    lines.push(`  --color-border: ${fallback('--color-border', '#3b424e')};`);
    lines.push(`  --color-border-strong: ${fallback('--color-border-strong', '#596270')};`);
    lines.push(`  --color-accent: ${fallback('--color-accent', '#c5cbd4')};`);
    lines.push(`  --color-accent-hover: ${fallback('--color-accent-hover', '#e0e4ea')};`);
    lines.push(`  --color-accent-active: ${fallback('--color-accent-active', '#ffffff')};`);
    lines.push(`  --color-accent-text: ${fallback('--color-accent-text', '#15181d')};`);
    lines.push(`  --color-focus-ring: ${fallback('--color-focus-ring', '#dde2e9')};`);
    lines.push(`  --color-selection: ${fallback('--color-selection', '#3a4655')};`);
    lines.push(`  --color-caret: ${fallback('--color-caret', '#f1f3f6')};`);
    lines.push(`  --color-typing-pending: ${fallback('--color-typing-pending', '#a7aeb9')};`);
    lines.push(`  --color-typing-current: ${fallback('--color-typing-current', '#ffffff')};`);
    lines.push(`  --color-typing-correct: ${fallback('--color-typing-correct', '#a9b8ae')};`);
    lines.push(`  --color-typing-incorrect: ${fallback('--color-typing-incorrect', '#e39a9a')};`);
    lines.push(`  --color-typing-corrected: ${fallback('--color-typing-corrected', '#d3a477')};`);
    lines.push(`  --color-key-background: ${fallback('--color-key-background', '#1b1f25')};`);
    lines.push(`  --color-key-border: ${fallback('--color-key-border', '#444b57')};`);
    lines.push(`  --color-key-active: ${fallback('--color-key-active', '#d9dee5')};`);
    lines.push(`  --color-key-pressed: ${fallback('--color-key-pressed', '#ffffff')};`);
    lines.push(`  --color-success: ${fallback('--color-success', '#9fbba7')};`);
    lines.push(`  --color-warning: ${fallback('--color-warning', '#d1af77')};`);
    lines.push(`  --color-error: ${fallback('--color-error', '#dc8d8d')};`);
    lines.push(`  --color-information: ${fallback('--color-information', '#c5cbd4')};`);
    lines.push(`  --color-chart-primary: ${fallback('--color-chart-primary', '#d7dce3')};`);
    lines.push(`  --color-chart-secondary: ${fallback('--color-chart-secondary', '#9aa4b1')};`);
    lines.push(`  --color-chart-positive: ${fallback('--color-chart-positive', '#9fbba7')};`);
    lines.push(`  --color-chart-negative: ${fallback('--color-chart-negative', '#dc8d8d')};`);
    lines.push(`  --color-chart-grid: ${fallback('--color-chart-grid', '#454c57')};`);
    lines.push(`  --color-chart-axis: ${fallback('--color-chart-axis', '#737c89')};`);
    lines.push(`  --color-chart-label: ${fallback('--color-chart-label', '#b9bec7')};`);
    lines.push(`  --color-chart-tooltip-background: ${fallback('--color-chart-tooltip-background', '#20242b')};`);
    lines.push(`  --color-chart-tooltip-border: ${fallback('--color-chart-tooltip-border', '#5c6572')};`);
    lines.push(`  --color-chart-selected: ${fallback('--color-chart-selected', '#ffffff')};`);
    lines.push(`  --color-progress-track: ${fallback('--color-progress-track', '#303641')};`);
    lines.push(`  --color-progress-fill: ${fallback('--color-progress-fill', '#c5cbd4')};`);
    lines.push(`  --color-overlay: ${fallback('--color-overlay', '#111419')};`);
    lines.push(`  --color-modal-surface: ${fallback('--color-modal-surface', '#1c2027')};`);
    lines.push(`  --color-tooltip-surface: ${fallback('--color-tooltip-surface', '#1c2027')};`);
    lines.push(`  --color-scrollbar: ${fallback('--color-scrollbar', '#3b424e')};`);
    lines.push(`  --color-scrollbar-hover: ${fallback('--color-scrollbar-hover', '#c5cbd4')};`);
    lines.push('  --shadow-surface: 0 1px 2px rgba(0, 0, 0, 0.28);');
    lines.push('  --shadow-elevated: 0 12px 28px rgba(0, 0, 0, 0.24);');
    lines.push('');
    lines.push('  --bg: var(--color-app-background);');
    lines.push('  --bg-sub: var(--color-surface-primary);');
    lines.push('  --main: var(--color-accent);');
    lines.push('  --sub: var(--color-text-secondary);');
    lines.push('  --text: var(--color-text-primary);');
    lines.push('  --error: var(--color-error);');
    lines.push('  --caret: var(--color-caret);');
    lines.push('}');
    return lines.join('\n');
  }

  async function applyTheme(name: string) {
    const styleEl = document.getElementById('theme-style') || (() => {
      const el = document.createElement('style');
      el.id = 'theme-style';
      document.head.appendChild(el);
      return el;
    })();
    styleEl.setAttribute('data-theme', name);

    const root = document.documentElement;
    for (const variable of appliedThemeVariables) {
      root.style.removeProperty(variable);
    }
    appliedThemeVariables.clear();

    if (name === 'custom') {
      // Кастомная тема живёт целиком на фронтенде: синтезируем CSS-переменные
      // из сохранённого JSON (включая производные legacy-алиасы).
      const css = buildCustomThemeCss(settings?.custom_theme_colors ?? '');
      styleEl.textContent = css;
      for (const match of css.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;{}]+)\s*;/g)) {
        root.style.setProperty(match[1], match[2].trim(), 'important');
        appliedThemeVariables.add(match[1]);
      }
      root.dataset.theme = name;
      root.style.colorScheme = 'dark';
      return;
    }

    const css = await ipc.getThemeCss(name);
    styleEl.textContent = css;

    // Apply variables inline as well as through the stylesheet. This keeps
    // theme switching reliable when component-scoped CSS is present and lets
    // semantic aliases such as --bg resolve to the active theme tokens.
    const variables = /(--[a-z0-9-]+)\s*:\s*([^;{}]+)\s*;/g;
    for (const match of css.matchAll(variables)) {
      const variable = match[1];
      root.style.setProperty(variable, match[2].trim(), 'important');
      appliedThemeVariables.add(variable);
    }
    root.dataset.theme = name;
    const themeInfo = themes.find((theme) => theme.name === name);
    root.style.colorScheme = themeInfo?.is_dark ? 'dark' : 'light';
  }

  async function selectTheme(name: string) {
    try {
      await applyTheme(name);
      await ipc.setSetting('theme', name);
      activeTheme = name;
      settings = await ipc.getSettings();
      deps.setError('');
    } catch (error) {
      const detail = error instanceof Error
        ? error.message
        : typeof error === 'object' && error !== null
          ? JSON.stringify(error)
          : String(error);
      deps.setError(`Theme error: ${detail}`);
      console.error('Theme switch failed', { name, error });
    }
  }

  async function updateSetting(key: string, value: unknown) {
    try {
      await ipc.setSetting(key, value);
      settings = await ipc.getSettings();
    } catch (error) {
      deps.setError(`Settings error: ${ipc.ipcErrorMessage(error)}`);
      return;
    }
    if (key === 'ui_language') {
      uiLang = (value as string) || 'en';
    }
  }

  return {
    get settings() { return settings; },
    get uiLang() { return uiLang; },
    get themes() { return themes; },
    get activeTheme() { return activeTheme; },
    get mainFontSize() { return mainFontSize; },
    loadThemes,
    loadSettings,
    applyTheme,
    selectTheme,
    updateSetting,
  };
}
