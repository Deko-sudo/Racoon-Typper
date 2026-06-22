<script lang="ts">
  import type { ModeName, LanguageCode } from '../types/index';

  let {
    selectedMode,
    selectedDuration,
    selectedWordCount,
    selectedLanguage,
    onSelectMode,
    onSelectDuration,
    onSelectWordCount,
    onSelectLanguage,
  }: {
    selectedMode: ModeName;
    selectedDuration: number;
    selectedWordCount: number;
    selectedLanguage: LanguageCode;
    onSelectMode: (m: ModeName) => void;
    onSelectDuration: (d: number) => void;
    onSelectWordCount: (w: number) => void;
    onSelectLanguage: (l: LanguageCode) => void;
  } = $props();
</script>

<div class="mode-selector">
  <div class="mode-group">
    <button class:active={selectedMode === 'time'} onclick={() => onSelectMode('time')}>Time</button>
    <button class:active={selectedMode === 'words'} onclick={() => onSelectMode('words')}>Words</button>
    <button class:active={selectedMode === 'quote'} onclick={() => onSelectMode('quote')}>Quote</button>
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
  <div class="lang-group">
    <button class:active={selectedLanguage === 'en'} onclick={() => onSelectLanguage('en')}>EN</button>
    <button class:active={selectedLanguage === 'ru'} onclick={() => onSelectLanguage('ru')}>RU</button>
  </div>
</div>

<style>
  .mode-selector { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; justify-content: center; }
  .mode-group, .preset-group, .lang-group { display: flex; gap: 0.25rem; }
  .mode-group button, .preset-group button, .lang-group button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.25rem 0.75rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
  }
  .mode-group button.active, .preset-group button.active, .lang-group button.active {
    color: var(--main); border-color: var(--main);
  }
</style>