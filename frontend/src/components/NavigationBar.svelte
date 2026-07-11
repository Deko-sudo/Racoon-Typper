<script lang="ts">
  import type { ViewName } from '../lib/types/index';
  import { t } from '../lib/i18n';

  let { view, historyTotal = 0, uiLang = 'en', onNavigate }: {
    view: ViewName;
    historyTotal?: number;
    uiLang?: string;
    onNavigate: (v: ViewName) => void;
  } = $props();
</script>

<nav aria-label="Primary navigation">
  <button aria-label={t(uiLang, 'nav.dashboard')} aria-current={view === 'dashboard' ? 'page' : undefined} class:active={view === 'dashboard'} onclick={() => onNavigate('dashboard')}>{t(uiLang, 'nav.dashboard')}</button>
  <button aria-label={t(uiLang, 'nav.test')} aria-current={view === 'test' ? 'page' : undefined} class:active={view === 'test'} onclick={() => onNavigate('test')}>{t(uiLang, 'nav.test')}</button>
  <button aria-label={t(uiLang, 'nav.lessons')} aria-current={view === 'lessons' ? 'page' : undefined} class:active={view === 'lessons'} onclick={() => onNavigate('lessons')}>{t(uiLang, 'nav.lessons')}</button>
  <button aria-label={t(uiLang, 'nav.weakkeys')} aria-current={view === 'weakkeys' ? 'page' : undefined} class:active={view === 'weakkeys'} onclick={() => onNavigate('weakkeys')}>{t(uiLang, 'nav.weakkeys')}</button>
  <button aria-label={t(uiLang, 'nav.analytics')} aria-current={view === 'analytics' ? 'page' : undefined} class:active={view === 'analytics'} onclick={() => onNavigate('analytics')}>{t(uiLang, 'nav.analytics')}</button>
  <button aria-label={t(uiLang, 'nav.achievements')} aria-current={view === 'achievements' ? 'page' : undefined} class:active={view === 'achievements'} onclick={() => onNavigate('achievements')}>{t(uiLang, 'nav.achievements')}</button>
  <button aria-label={`${t(uiLang, 'nav.history')} (${historyTotal})`} aria-current={view === 'history' ? 'page' : undefined} class:active={view === 'history'} onclick={() => onNavigate('history')}>{t(uiLang, 'nav.history')} ({historyTotal})</button>
  <button aria-label={t(uiLang, 'nav.bests')} aria-current={view === 'bests' ? 'page' : undefined} class:active={view === 'bests'} onclick={() => onNavigate('bests')}>{t(uiLang, 'nav.bests')}</button>
  <button aria-label={t(uiLang, 'nav.texts')} aria-current={view === 'custom' ? 'page' : undefined} class:active={view === 'custom'} onclick={() => onNavigate('custom')}>{t(uiLang, 'nav.texts')}</button>
  <button aria-label={t(uiLang, 'nav.settings')} aria-current={view === 'settings' ? 'page' : undefined} class:active={view === 'settings'} onclick={() => onNavigate('settings')}>{t(uiLang, 'nav.settings')}</button>
</nav>

<style>
  nav { display: flex; gap: 0.5rem; flex-wrap: wrap; justify-content: center; }
  nav button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.5rem 1rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px;
  }
  nav button.active { color: var(--main); border-color: var(--main); }
</style>
