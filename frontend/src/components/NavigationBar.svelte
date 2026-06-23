<script lang="ts">
  import type { ViewName } from '../types/index';

  let { view, historyTotal = 0, onNavigate }: { view: ViewName; historyTotal?: number; onNavigate: (v: ViewName) => void } = $props();
</script>

<nav>
  <button class:active={view === 'dashboard'} onclick={() => onNavigate('dashboard')}>Dashboard</button>
  <button class:active={view === 'test'} onclick={() => onNavigate('test')}>Test</button>
    <button class:active={view === 'lessons'} onclick={() => onNavigate('lessons')}>Lessons</button>
    <button class:active={view === 'weakkeys'} onclick={() => onNavigate('weakkeys')}>Weak Keys</button>
    <button class:active={view === 'history'} onclick={() => onNavigate('history')}>History ({historyTotal})</button>
  <button class:active={view === 'bests'} onclick={() => onNavigate('bests')}>Bests</button>
  <button class:active={view === 'custom'} onclick={() => onNavigate('custom')}>Texts</button>
  <button class:active={view === 'settings'} onclick={() => onNavigate('settings')}>Settings</button>
</nav>

<style>
  nav { display: flex; gap: 0.5rem; }
  nav button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.5rem 1rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px;
  }
  nav button.active { color: var(--main); border-color: var(--main); }
</style>