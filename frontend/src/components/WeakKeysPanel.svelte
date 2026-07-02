<script lang="ts">
  import KeyboardVizComponent from './KeyboardVizComponent.svelte';
  import type { CharStatus } from '../lib/types/index';
  import { t } from '../lib/i18n';

  let {
    weakKeys = [],
    charStats = {},
    onGenerateTraining,
    uiLang = 'en',
    trainingText = '',
    trainingCharStatuses = [] as CharStatus[],
    trainingCaretPos = 0,
    trainingRunning = false,
  }: {
    weakKeys: Array<{ ch: string; error_count: number; accuracy: number; rank: number }>;
    charStats: Record<string, { correct: number; incorrect: number; total: number }>;
    onGenerateTraining: () => void;
    uiLang?: string;
    trainingText?: string;
    trainingCharStatuses?: CharStatus[];
    trainingCaretPos?: number;
    trainingRunning?: boolean;
  } = $props();

  // Viewport — same as TestView
  const VIEWPORT_CHARS = 120;
  const VIEWPORT_PADDING = 30;

  let viewportStart = $derived(Math.max(0, trainingCaretPos - VIEWPORT_PADDING));
  let viewportEnd = $derived(Math.min(trainingCharStatuses.length, viewportStart + VIEWPORT_CHARS));
  let viewportChars = $derived(trainingCharStatuses.slice(viewportStart, viewportEnd));
  let viewportOffset = $derived(trainingCaretPos - viewportStart);

  function charClass(char: CharStatus, idx: number): string {
    const classes: string[] = [char.status];
    if (idx < viewportOffset) classes.push('past');
    else if (idx === viewportOffset) classes.push('current');
    else classes.push('future');
    return classes.join(' ');
  }
</script>

<div class="weak-keys-panel">
  <h3>{t(uiLang, 'weakkeys.title')}</h3>

  {#if weakKeys.length === 0}
    <p class="empty">{t(uiLang, 'weakkeys.empty')}</p>
  {:else}
    <div class="weak-keys-list">
      {#each weakKeys as wk}
        <div class="weak-key-row" class:critical={wk.accuracy < 70}>
          <span class="wk-char">{wk.ch}</span>
          <span class="wk-accuracy">{wk.accuracy.toFixed(1)}%</span>
          <span class="wk-errors">{wk.error_count} {t(uiLang, 'weakkeys.errors')}</span>
          <span class="wk-rank">#{wk.rank}</span>
        </div>
      {/each}
    </div>
    <button onclick={onGenerateTraining}>{t(uiLang, 'weakkeys.generate')}</button>
  {/if}

  {#if trainingText && trainingRunning}
    <div class="text-viewport">
      <div class="text-display">
        {#if viewportStart > 0}<span class="text-ellipsis">…</span>{/if}
        {#each viewportChars as char, i}
          <span class="char {charClass(char, i)}" class:caret={i === viewportOffset}>
            {char.expected === ' ' ? '\u00A0' : char.expected}
          </span>
        {/each}
        {#if viewportEnd < trainingCharStatuses.length}<span class="text-ellipsis">…</span>{/if}
      </div>
    </div>
  {/if}

  <KeyboardVizComponent {charStats} />
</div>

<style>
  .weak-keys-panel { max-width: 1200px; width: 100%; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; }
  .empty { color: var(--sub); text-align: center; padding: 1rem; }
  .weak-keys-list { display: flex; flex-direction: column; gap: 0.25rem; margin-bottom: 1rem; }
  .weak-key-row {
    display: flex; align-items: center; gap: 1rem; padding: 0.5rem 1rem;
    background: var(--bg-sub); border-radius: 4px; border: 1px solid var(--sub);
  }
  .weak-key-row.critical { border-color: var(--error); }
  .wk-char { font-weight: bold; color: var(--main); min-width: 1.5rem; }
  .wk-accuracy { color: var(--text); min-width: 4rem; }
  .wk-errors { color: var(--sub); font-size: 0.75rem; flex: 1; }
  .wk-rank { color: var(--sub); font-size: 0.75rem; }
  button {
    background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main);
    padding: 0.5rem 1.5rem; font-family: inherit; font-size: 0.875rem;
    cursor: pointer; border-radius: 4px; margin-bottom: 1rem;
  }
  button:hover { background-color: var(--main); color: var(--bg); }

  .text-viewport {
    max-width: 1200px; width: 100%; overflow: hidden;
    background-color: var(--bg-sub); border-radius: 8px;
    padding: 2rem 1.5rem; position: relative; margin-bottom: 1rem;
  }
  .text-display {
    font-size: 2rem; line-height: 1.8; text-align: center;
    user-select: none; white-space: pre-wrap; word-wrap: break-word;
    min-height: 3.6em; display: flex; flex-wrap: wrap; justify-content: center; align-items: center;
  }
  .text-ellipsis { color: var(--sub); opacity: 0.4; padding: 0 0.25rem; }
  .char { transition: color 0.05s, opacity 0.1s; position: relative; }
  .char.pending { color: var(--sub); }
  .char.correct { color: var(--text); }
  .char.incorrect { color: var(--error); }
  .char.backspaced { color: #ff8c42; }
  .char.past { opacity: 0.5; }
  .char.current { opacity: 1; font-weight: 600; }
  .char.future { opacity: 0.35; }
  .char.caret::before {
    content: '|'; position: absolute; left: -0.5ch; color: var(--caret);
    animation: blink 1s infinite;
  }
  @keyframes blink { 0%,50%{opacity:1} 51%,100%{opacity:0} }
</style>