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

  // Viewport: показываем окно из N символов вокруг курсора
  const VIEWPORT_CHARS = 80;
  const VIEWPORT_PADDING = 20;

  let viewportStart = $derived(Math.max(0, caretPos - VIEWPORT_PADDING));
  let viewportEnd = $derived(Math.min(charStatuses.length, viewportStart + VIEWPORT_CHARS));
  let viewportChars = $derived(charStatuses.slice(viewportStart, viewportEnd));
  let viewportOffset = $derived(caretPos - viewportStart);
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
  <div class="text-viewport">
    <div class="text-display">
      {#if viewportStart > 0}<span class="text-ellipsis">…</span>{/if}
      {#each viewportChars as char, i}
        <span
          class="char {char.status}"
          class:caret={i === viewportOffset}
        >{char.expected === ' ' ? '\u00A0' : char.expected}</span>
      {/each}
      {#if viewportEnd < charStatuses.length}<span class="text-ellipsis">…</span>{/if}
    </div>
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
  .text-viewport {
    max-width: 900px; width: 100%; overflow: hidden;
    background-color: var(--bg-sub); border-radius: 8px;
    padding: 2rem 1.5rem; position: relative;
  }
  .text-display {
    font-size: 2rem; line-height: 1.8; text-align: center;
    user-select: none; white-space: pre-wrap; word-wrap: break-word;
    min-height: 3.6em; display: flex; flex-wrap: wrap; justify-content: center; align-items: center;
  }
  .text-ellipsis { color: var(--sub); opacity: 0.5; padding: 0 0.25rem; }
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