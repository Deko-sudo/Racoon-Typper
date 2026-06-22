<script lang="ts">
  import type { CharStatus, AppSettings } from '../types/index';
  import ModeSelector from './ModeSelector.svelte';
  import ResultOverlay from './ResultOverlay.svelte';
  import type { ModeName, LanguageCode, FinalStats } from '../types/index';

  let {
    text,
    caretPos,
    charStatuses,
    isRunning,
    isComplete,
    liveWpm,
    liveAccuracy,
    elapsedMs,
    finalStats,
    settings,
    selectedMode,
    selectedDuration,
    selectedWordCount,
    selectedLanguage,
    sessionModeType,
    sessionLanguage,
    onModeChange,
    onDurationChange,
    onWordCountChange,
    onLanguageChange,
    onAbort,
    onRestart,
  }: {
    text: string;
    caretPos: number;
    charStatuses: CharStatus[];
    isRunning: boolean;
    isComplete: boolean;
    liveWpm: number;
    liveAccuracy: number;
    elapsedMs: number;
    finalStats: FinalStats | null;
    settings: AppSettings | null;
    selectedMode: ModeName;
    selectedDuration: number;
    selectedWordCount: number;
    selectedLanguage: LanguageCode;
    sessionModeType: string;
    sessionLanguage: string;
    onModeChange: (m: ModeName) => void;
    onDurationChange: (d: number) => void;
    onWordCountChange: (w: number) => void;
    onLanguageChange: (l: LanguageCode) => void;
    onAbort: () => void;
    onRestart: () => void;
  } = $props();
</script>

{#if isComplete && finalStats}
  <ResultOverlay stats={finalStats} onRestart={onRestart} />
{:else if text}
  <ModeSelector
    {selectedMode}
    {selectedDuration}
    {selectedWordCount}
    {selectedLanguage}
    onSelectMode={onModeChange}
    onSelectDuration={onDurationChange}
    onSelectWordCount={onWordCountChange}
    onSelectLanguage={onLanguageChange}
  />
  <div class="live-stats">
    {#if settings?.show_live_wpm}<span class="stat">WPM: {liveWpm.toFixed(0)}</span>{/if}
    {#if settings?.show_accuracy}<span class="stat">Acc: {liveAccuracy.toFixed(1)}%</span>{/if}
    <span class="stat">Time: {(elapsedMs / 1000).toFixed(1)}s</span>
    <span class="stat mode-badge">{sessionModeType}/{sessionLanguage}</span>
  </div>
  <div class="text-display" tabindex="0">
    {#each charStatuses as char, i}
      <span class="char {char.status}" class:caret={i === caretPos}>
        {char.expected === ' ' ? '\u00A0' : char.expected}
      </span>
    {/each}
  </div>
  <div class="info">
    <span>Position: {caretPos}/{text.length}</span>
    <button class="abort-btn" onclick={onAbort}>Abort</button>
  </div>
{/if}

<style>
  .live-stats { display: flex; gap: 2rem; font-size: 1.25rem; }
  .stat { color: var(--sub); }
  .mode-badge { font-size: 0.75rem; color: var(--main); }
  .text-display {
    font-size: 2rem; line-height: 1.8; max-width: 900px; text-align: center;
    padding: 2rem; background-color: var(--bg-sub); border-radius: 8px; user-select: none;
  }
  .char { transition: color 0.05s; position: relative; }
  .char.pending { color: var(--sub); }
  .char.correct { color: var(--text); }
  .char.incorrect { color: var(--error); }
  .char.backspaced { color: var(--sub); }
  .char.caret::before {
    content: '|'; position: absolute; left: -0.5ch; color: var(--caret);
    animation: blink 1s infinite;
  }
  @keyframes blink { 0%,50%{opacity:1} 51%,100%{opacity:0} }
  .info { display: flex; align-items: center; gap: 2rem; font-size: 0.875rem; color: var(--sub); }
  .abort-btn {
    background-color: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.25rem 1rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
  }
  .abort-btn:hover { background: var(--sub); color: var(--bg); }
</style>