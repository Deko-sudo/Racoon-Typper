<script lang="ts">
  import type { CharStatus, AppSettings } from '../lib/types/index';
  import ModeSelector from './ModeSelector.svelte';
  import ResultOverlay from './ResultOverlay.svelte';
  import KeyboardTrainer from './KeyboardTrainer.svelte';
  import HandPositionGuide from './HandPositionGuide.svelte';
  import type { ModeName, LanguageCode, FinalStats } from '../lib/types/index';
  import { t } from '../lib/i18n';

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
    uiLang = 'en',
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
    uiLang?: string;
  } = $props();

  // Viewport: окно из N символов вокруг курсора
  const VIEWPORT_CHARS = 120;
  const VIEWPORT_PADDING = 30;

  let viewportStart = $derived(Math.max(0, caretPos - VIEWPORT_PADDING));
  let viewportEnd = $derived(Math.min(charStatuses.length, viewportStart + VIEWPORT_CHARS));
  let viewportChars = $derived(charStatuses.slice(viewportStart, viewportEnd));
  let viewportOffset = $derived(caretPos - viewportStart);

  // Next character for keyboard trainer
  let nextChar = $derived(isRunning && !isComplete && caretPos < text.length ? text[caretPos] : '');
  let isRussian = $derived(sessionLanguage === 'ru');

  // Character focus classes
  function charClass(char: CharStatus, idx: number): string {
    const classes: string[] = [char.status];
    if (idx < viewportOffset) classes.push('past');
    else if (idx === viewportOffset) classes.push('current');
    else classes.push('future');
    return classes.join(' ');
  }
</script>

{#if isComplete && finalStats}
  <ResultOverlay stats={finalStats} onRestart={onRestart} {uiLang} />
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
    {uiLang}
  />
  <div class="live-stats">
    {#if settings?.show_live_wpm}<span class="stat">{t(uiLang, 'test.wpm')}: {liveWpm.toFixed(0)}</span>{/if}
    {#if settings?.show_accuracy}<span class="stat">{t(uiLang, 'test.acc')}: {liveAccuracy.toFixed(1)}%</span>{/if}
    <span class="stat">{t(uiLang, 'test.time')}: {(elapsedMs / 1000).toFixed(1)}s</span>
    <span class="stat mode-badge">{sessionModeType}/{sessionLanguage}</span>
  </div>

  <div class="text-viewport">
    <div class="text-display">
      {#if viewportStart > 0}<span class="text-ellipsis">…</span>{/if}
      {#each viewportChars as char, i}
        <span class="char {charClass(char, i)}" class:caret={i === viewportOffset}>
          {char.expected === ' ' ? '\u00A0' : char.expected}
        </span>
      {/each}
      {#if viewportEnd < charStatuses.length}<span class="text-ellipsis">…</span>{/if}
    </div>
  </div>

  <div class="info">
    <span>{t(uiLang, 'test.position')}: {caretPos}/{text.length}</span>
    <button class="abort-btn" onclick={onAbort}>{t(uiLang, 'test.abort')}</button>
  </div>

  {#if settings?.show_keyboard_trainer && isRunning}
    <KeyboardTrainer {nextChar} {isRussian} />
  {/if}

  {#if settings?.show_hand_guide && isRunning}
    <HandPositionGuide {nextChar} {isRussian} />
  {/if}
{/if}

<style>
  .live-stats { display: flex; gap: 2rem; font-size: 1.25rem; }
  .stat { color: var(--sub); }
  .mode-badge { font-size: 0.75rem; color: var(--main); }
  .text-viewport {
    max-width: 1200px; width: 100%; overflow: hidden;
    background-color: var(--bg-sub); border-radius: 8px;
    padding: 2rem 1.5rem; position: relative;
  }
  .text-display {
    font-size: 2rem; line-height: 1.8; text-align: center;
    user-select: none; white-space: pre-wrap; word-wrap: break-word;
    min-height: 3.6em; display: flex; flex-wrap: wrap; justify-content: center; align-items: center;
    transition: transform 0.15s ease-out;
  }
  .text-ellipsis { color: var(--sub); opacity: 0.4; padding: 0 0.25rem; }
  .char { transition: color 0.05s, opacity 0.1s; position: relative; }
  .char.pending { color: var(--sub); }
  .char.correct { color: var(--text); }
  .char.incorrect { color: var(--error); }
  .char.backspaced { color: #ff8c42; }
  /* Character focus: past = dimmer, current = bright, future = grey */
  .char.past { opacity: 0.5; }
  .char.current { opacity: 1; font-weight: 600; }
  .char.future { opacity: 0.35; }
  .char.caret::before {
    content: '|'; position: absolute; left: -0.5ch; color: var(--caret);
    animation: blink 1s infinite;
  }
  @keyframes blink { 0%,50%{opacity:1} 51%,100%{opacity:0} }
  .char.correct { transition: color 0.1s ease; }
  .char.incorrect { animation: shake 0.2s; }
  @keyframes shake { 0%,100%{transform:translateX(0)} 25%{transform:translateX(-2px)} 75%{transform:translateX(2px)} }
  .info { display: flex; align-items: center; gap: 2rem; font-size: 0.875rem; color: var(--sub); }
  .abort-btn {
    background-color: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.25rem 1rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
  }
  .abort-btn:hover { background: var(--sub); color: var(--bg); }
</style>