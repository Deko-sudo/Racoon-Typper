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

<nav>
  <button class:active={view === 'dashboard'} onclick={() => onNavigate('dashboard')}>{t(uiLang, 'nav.dashboard')}</button>
  <button class:active={view === 'test'} onclick={() => onNavigate('test')}>{t(uiLang, 'nav.test')}</button>
  <button class:active={view === 'lessons'} onclick={() => onNavigate('lessons')}>{t(uiLang, 'nav.lessons')}</button>
  <button class:active={view === 'weakkeys'} onclick={() => onNavigate('weakkeys')}>{t(uiLang, 'nav.weakkeys')}</button>
  <button class:active={view === 'analytics'} onclick={() => onNavigate('analytics')}>{t(uiLang, 'nav.analytics')}</button>
  <button class:active={view === 'achievements'} onclick={() => onNavigate('achievements')}>{t(uiLang, 'nav.achievements')}</button>
  <button class:active={view === 'history'} onclick={() => onNavigate('history')}>{t(uiLang, 'nav.history')} ({historyTotal})</button>
  <button class:active={view === 'bests'} onclick={() => onNavigate('bests')}>{t(uiLang, 'nav.bests')}</button>
  <button class:active={view === 'custom'} onclick={() => onNavigate('custom')}>{t(uiLang, 'nav.texts')}</button>
  <button class:active={view === 'settings'} onclick={() => onNavigate('settings')}>{t(uiLang, 'nav.settings')}</button>
</nav>

<style>
  nav { display: flex; gap: 0.5rem; flex-wrap: wrap; justify-content: center; }
  nav button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.5rem 1rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px;
  }
  nav button.active { color: var(--main); border-color: var(--main); }
</style>