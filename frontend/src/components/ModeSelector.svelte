<script lang="ts">
  import type { ModeName, LanguageCode } from '../lib/types/index';
  import { t } from '../lib/i18n';

  let {
    selectedMode,
    selectedDuration,
    selectedWordCount,
    selectedLanguage,
    onSelectMode,
    onSelectDuration,
    onSelectWordCount,
    onSelectLanguage,
    uiLang = 'en',
  }: {
    selectedMode: ModeName;
    selectedDuration: number;
    selectedWordCount: number;
    selectedLanguage: LanguageCode;
    onSelectMode: (m: ModeName) => void;
    onSelectDuration: (d: number) => void;
    onSelectWordCount: (w: number) => void;
    onSelectLanguage: (l: LanguageCode) => void;
    uiLang?: string;
  } = $props();

  let langOpen = $state(false);

  const LANGS: [string, string][] = [
    ['en','EN'],['ru','RU'],['de','DE'],['uk','UK'],['cs','CS'],['pl','PL'],
    ['ro','RO'],['it','IT'],['fr','FR'],['es','ES'],['pt','PT'],['ja','JA'],
    ['zh-hk','繁HK'],['zh-tw','繁TW'],['ko','KO'],
  ];

  const currentLabel = LANGS.find(([c]) => c === selectedLanguage)?.[1] || 'EN';
</script>

<div class="mode-selector">
  <div class="mode-group">
    <button class:active={selectedMode === 'time'} onclick={() => onSelectMode('time')}>{t(uiLang, 'mode.time')}</button>
    <button class:active={selectedMode === 'words'} onclick={() => onSelectMode('words')}>{t(uiLang, 'mode.words')}</button>
    <button class:active={selectedMode === 'quote'} onclick={() => onSelectMode('quote')}>{t(uiLang, 'mode.quote')}</button>
  </div>
  {#if selectedMode === 'time'}
    <div class="preset-group">
      {#each [15, 30, 60, 120] as d}
        <button class:active={selectedDuration === d} onclick={() => onSelectDuration(d)}>{d}s</button>
      {/each}
    </div>
  {/if}
  {#if selectedMode === 'words'}
    <div class="preset-group">
      {#each [10, 25, 50, 100] as w}
        <button class:active={selectedWordCount === w} onclick={() => onSelectWordCount(w)}>{w}</button>
      {/each}
    </div>
  {/if}
  <div class="lang-dropdown">
    <button class="lang-current" onclick={() => langOpen = !langOpen}>
      {currentLabel}
      <span class="arrow {langOpen ? 'open' : ''}">▾</span>
    </button>
    {#if langOpen}
      <div class="lang-list">
        {#each LANGS as [code, label]}
          <button
            class:active={selectedLanguage === code}
            onclick={() => { onSelectLanguage(code as LanguageCode); langOpen = false; }}
          >{label}</button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .mode-selector { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; justify-content: center; position: relative; }
  .mode-group, .preset-group { display: flex; gap: 0.25rem; }
  .mode-group button, .preset-group button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.25rem 0.75rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
  }
  .mode-group button.active, .preset-group button.active {
    color: var(--main); border-color: var(--main);
  }
  .lang-dropdown { position: relative; }
  .lang-current {
    background: var(--bg-sub); color: var(--main); border: 1px solid var(--main);
    padding: 0.25rem 0.75rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
    display: flex; align-items: center; gap: 0.25rem;
  }
  .arrow { font-size: 0.625rem; transition: transform 0.2s; }
  .arrow.open { transform: rotate(180deg); }
  .lang-list {
    position: absolute; top: 100%; left: 0; z-index: 10;
    display: flex; flex-wrap: wrap; gap: 0.25rem; max-width: 280px;
    background: var(--bg-sub); border: 1px solid var(--sub); border-radius: 6px;
    padding: 0.5rem; margin-top: 0.25rem;
  }
  .lang-list button {
    background: transparent; color: var(--sub); border: 1px solid transparent;
    padding: 0.2rem 0.5rem; font-family: inherit; font-size: 0.7rem; cursor: pointer; border-radius: 3px;
  }
  .lang-list button:hover { background: var(--bg); color: var(--text); }
  .lang-list button.active { color: var(--main); border-color: var(--main); }
</style>