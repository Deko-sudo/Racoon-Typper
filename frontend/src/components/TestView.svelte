<script lang="ts">
  import type { CharStatus, AppSettings } from '../lib/types/index';
  import ModeSelector from './ModeSelector.svelte';
  import ResultOverlay from './ResultOverlay.svelte';
  import KeyboardTrainer from './KeyboardTrainer.svelte';
  import HandPositionGuide from './HandPositionGuide.svelte';
  import type { ModeName, LanguageCode, FinalStats } from '../lib/types/index';
  import { t } from '../lib/i18n';
  import { VIEWPORT_CHARS, VIEWPORT_PADDING } from '../lib/keyboard';

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

  let viewportStart = $derived(Math.max(0, caretPos - VIEWPORT_PADDING));
  let viewportEnd = $derived(Math.min(charStatuses.length, viewportStart + VIEWPORT_CHARS));
  let viewportChars = $derived(charStatuses.slice(viewportStart, viewportEnd));
  let viewportOffset = $derived(caretPos - viewportStart);
  let progress = $derived(charStatuses.length === 0 ? 0 : (caretPos / charStatuses.length) * 100);
  let nextChar = $derived(isRunning && !isComplete && caretPos < text.length ? text[caretPos] : '');
  let isRussian = $derived(sessionLanguage === 'ru');

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

  <section class="typing-card" aria-label={t(uiLang, 'test.typing_title')}>
    <header class="typing-header">
      <div><p class="typing-eyebrow">{t(uiLang, 'test.typing_label')}</p><h3>{t(uiLang, 'test.typing_title')}</h3></div>
      <div class="typing-progress"><span>{caretPos}/{text.length}</span><div class="progress-track"><div class="progress-fill" style:width={`${progress}%`}></div></div></div>
    </header>
    <div class="text-viewport"><div class="text-display">
      {#if viewportStart > 0}<span class="text-ellipsis">…</span>{/if}
      {#each viewportChars as char, i}
        <span class="char {charClass(char, i)}" class:caret={i === viewportOffset}>{char.expected === ' ' ? '\u00A0' : char.expected}</span>
      {/each}
      {#if viewportEnd < charStatuses.length}<span class="text-ellipsis">…</span>{/if}
    </div></div>
  </section>

  <div class="info">
    <button class="abort-btn" onclick={onAbort}>{t(uiLang, 'test.abort')}</button>
  </div>

  {#if isRunning}
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
  .typing-card { max-width: 1200px; width: 100%; margin:1.5rem 0; padding: 1.25rem; border: 1px solid color-mix(in srgb, var(--main) 38%, var(--sub)); border-radius: 12px; background: linear-gradient(145deg, color-mix(in srgb, var(--bg-sub) 88%, var(--main)), var(--bg-sub)); box-shadow: 0 12px 28px rgba(0,0,0,0.16); }
  .typing-header { display: flex; justify-content: space-between; align-items: end; gap: 1rem; margin-bottom: 1rem; }
  .typing-eyebrow { color: var(--main); text-transform: uppercase; letter-spacing: .08em; font-size: .62rem; font-weight: 700; }
  .typing-header h3 { margin: .15rem 0 0; color: var(--text); font-size: .95rem; }
  .typing-progress { min-width: 180px; color: var(--sub); font-size: .72rem; text-align: right; }
  .progress-track { width: 100%; height: 5px; overflow: hidden; border-radius: 999px; background: color-mix(in srgb, var(--sub) 45%, transparent); margin-top: .35rem; }
  .progress-fill { height: 100%; border-radius: inherit; background: var(--main); transition: width .12s ease; }
  .text-viewport { width:100%; overflow:hidden; background:color-mix(in srgb, var(--bg) 55%, var(--bg-sub)); border:1px solid var(--sub); border-radius:8px; padding:1.5rem; }
  .text-display { --typing-font-size:clamp(1.1rem,1.8vw,1.5rem); max-width:min(900px, 100%); margin:0 auto; font-size:0; line-height:1.65; letter-spacing:normal; text-align:center; user-select:none; white-space:pre-wrap; overflow-wrap:break-word; min-height:3.3em; display:block; }
  .text-ellipsis { color:var(--sub); opacity:.4; padding:0 .25rem; font-size:var(--typing-font-size); }
  .char { position:relative; display:inline-block; vertical-align:baseline; font-size:var(--typing-font-size); line-height:1.65; transition:color .05s, opacity .1s; }
  .char.pending { color:var(--color-typing-pending); } .char.correct { color:var(--color-typing-correct); } .char.incorrect { color:var(--color-typing-incorrect); animation:shake .2s; } .char.backspaced { color:var(--color-typing-corrected); }
  .char.past { opacity:.5; } .char.current { opacity:1; font-weight:600; } .char.current.pending { color:var(--color-typing-current); } .char.future { opacity:.35; }
  .char.caret::before { content:''; position:absolute; left:-.16em; top:.14em; bottom:.14em; width:.1em; border-radius:999px; background:var(--color-caret); box-shadow:0 0 .5em color-mix(in srgb,var(--color-caret) 70%,transparent); animation:blink .9s ease-in-out infinite; }
  @keyframes blink { 0%,45%{opacity:1} 55%,100%{opacity:.18} } @keyframes shake { 0%,100%{transform:translateX(0)} 25%{transform:translateX(-2px)} 75%{transform:translateX(2px)} }
  .info { display: flex; align-items: center; gap: 2rem; font-size: 0.875rem; color: var(--sub); }
  .abort-btn {
    background-color: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.25rem 1rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
  }
  .abort-btn:hover { background: var(--sub); color: var(--bg); }
</style>
