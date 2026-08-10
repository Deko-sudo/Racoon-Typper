<script lang="ts">
  import KeyboardTrainer from './KeyboardTrainer.svelte';
  import type { CharStatus } from '../lib/types/index';
  import { t } from '../lib/i18n';
  const TRAINING_VIEWPORT_CHARS = 180;
  const TRAINING_VIEWPORT_PADDING = 45;

  let {
    weakKeys = [],
    charStats = {},
    onGenerateTraining,
    uiLang = 'en',
    trainingText = '',
    trainingCharStatuses = [] as CharStatus[],
    trainingCaretPos = 0,
    trainingRunning = false,
    trainingLanguage = 'en',
  }: {
    weakKeys: Array<{ ch: string; error_count: number; accuracy: number; rank: number }>;
    charStats: Record<string, { correct: number; incorrect: number; total: number }>;
    onGenerateTraining: () => void;
    uiLang?: string;
    trainingText?: string;
    trainingCharStatuses?: CharStatus[];
    trainingCaretPos?: number;
    trainingRunning?: boolean;
    trainingLanguage?: string;
  } = $props();

  let viewportStart = $derived(Math.max(0, trainingCaretPos - TRAINING_VIEWPORT_PADDING));
  let viewportEnd = $derived(Math.min(trainingCharStatuses.length, viewportStart + TRAINING_VIEWPORT_CHARS));
  let viewportChars = $derived(trainingCharStatuses.slice(viewportStart, viewportEnd));
  let viewportOffset = $derived(trainingCaretPos - viewportStart);
  let progress = $derived(trainingCharStatuses.length === 0 ? 0 : (trainingCaretPos / trainingCharStatuses.length) * 100);
  let nextChar = $derived(trainingRunning && trainingCaretPos < trainingText.length ? trainingText[trainingCaretPos] : '');
  function charClass(char: CharStatus, idx: number): string { const classes: string[] = [char.status]; if (idx < viewportOffset) classes.push('past'); else if (idx === viewportOffset) classes.push('current'); else classes.push('future'); return classes.join(' '); }
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
    <section class="training-card"><header class="training-header"><div><p class="training-eyebrow">{t(uiLang, 'weakkeys.training_label')}</p><h4>{t(uiLang, 'weakkeys.training_title')}</h4></div><div class="training-progress"><span>{trainingCaretPos}/{trainingCharStatuses.length}</span><div class="progress-track"><div class="progress-fill" style:width={`${progress}%`}></div></div></div></header><div class="text-viewport"><div class="text-display">{#if viewportStart > 0}<span class="text-ellipsis">…</span>{/if}{#each viewportChars as char, i}<span class="char {charClass(char, i)}" class:caret={i === viewportOffset}>{char.expected === ' ' ? '\u00A0' : char.expected}</span>{/each}{#if viewportEnd < trainingCharStatuses.length}<span class="text-ellipsis">…</span>{/if}</div></div></section>
  {/if}

  <KeyboardTrainer {charStats} {nextChar} isRussian={trainingLanguage === 'ru'} />
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
  button:hover { background-color: var(--main); color: var(--color-accent-text); }
  .training-card { margin:1.5rem 0; padding:1.25rem; border:1px solid color-mix(in srgb,var(--main) 38%,var(--sub)); border-radius:12px; background:var(--bg-sub); }
  .training-header { display:flex; justify-content:space-between; align-items:end; gap:1rem; margin-bottom:1rem; } .training-eyebrow { color:var(--main); text-transform:uppercase; letter-spacing:.08em; font-size:.62rem; font-weight:700; } .training-header h4 { margin:.15rem 0 0; color:var(--text); font-size:.95rem; } .training-progress { min-width:180px; color:var(--sub); font-size:.72rem; text-align:right; }
  .progress-track { width:100%; height:5px; overflow:hidden; border-radius:999px; background:color-mix(in srgb,var(--sub) 45%,transparent); margin-top:.35rem; } .progress-fill { height:100%; border-radius:inherit; background:var(--main); transition:width .12s ease; }
  .text-viewport { width:100%; overflow:hidden; background:color-mix(in srgb,var(--bg) 55%,var(--bg-sub)); border:1px solid var(--sub); border-radius:8px; padding:clamp(1.25rem,3vw,2rem); } .text-display { font-size:clamp(1.5rem,3.5vw,2.25rem); line-height:1.8; text-align:center; user-select:none; white-space:pre-wrap; word-wrap:break-word; min-height:5.4em; display:flex; flex-wrap:wrap; justify-content:center; align-items:center; } .text-ellipsis { color:var(--sub); opacity:.4; padding:0 .25rem; } .char { position:relative; transition:color .05s,opacity .1s; } .char.pending { color:var(--color-typing-pending); } .char.correct { color:var(--color-typing-correct); } .char.incorrect { color:var(--color-typing-incorrect); } .char.backspaced { color:var(--color-typing-corrected); } .char.past { opacity:.5; } .char.current { opacity:1; font-weight:600; } .char.current.pending { color:var(--color-typing-current); } .char.future { opacity:.35; }
  .char.caret::before { content:''; position:absolute; left:-.16em; top:.14em; bottom:.14em; width:.1em; border-radius:999px; background:var(--color-caret); box-shadow:0 0 .5em color-mix(in srgb,var(--color-caret) 70%,transparent); animation:blink .9s ease-in-out infinite; } @keyframes blink { 0%,45%{opacity:1} 55%,100%{opacity:.18} }

</style>
