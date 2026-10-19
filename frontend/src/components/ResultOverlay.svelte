<script lang="ts">
  import type { FinalStats } from '../lib/types/index';
  import { renderShareCard } from '../lib/shareCard';
  import { saveResultCard, type CardExportOutcome } from '../lib/api/ipc';
  import KeyboardHeatmap from './KeyboardHeatmap.svelte';
  import { t } from '../lib/i18n';
  import type { LessonResultNavigation } from '../lib/lessonNavigation';

  let {
    stats,
    onRestart,
    lessonNavigation,
    onRepeatLesson,
    onNextLesson,
    onReturnToLessons,
    sessionModeType = '',
    sessionLanguage = '',
    keyboardLayout = 'qwerty',
    uiLang = 'en',
  }: {
    stats: FinalStats;
    onRestart: () => void;
    lessonNavigation: LessonResultNavigation | null;
    onRepeatLesson: () => void;
    onNextLesson: () => void;
    onReturnToLessons: () => void;
    sessionModeType?: string;
    sessionLanguage?: string;
    keyboardLayout?: string;
    uiLang?: string;
  } = $props();

  let shareState = $state<'idle' | 'rendering' | 'saving' | 'done' | 'error'>('idle');

  let shareOutcome = $state<CardExportOutcome | null>(null);

  function themeCssVar(name: string, fallback: string): string {
    const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
  }

  async function handleSharePng() {
    if (shareState === 'rendering' || shareState === 'saving') return;
    shareState = 'rendering';
    shareOutcome = null;
    try {
      const colors = {
        background: themeCssVar('--color-app-background', '#0d0f12'),
        surface: themeCssVar('--color-surface-primary', '#15181d'),
        text: themeCssVar('--color-text-primary', '#e7e9ed'),
        sub: themeCssVar('--color-text-secondary', '#8c94a0'),
        accent: themeCssVar('--color-accent', '#c5cbd4'),
        error: themeCssVar('--color-error', '#dc8d8d'),
      };
      const heatmap = (stats.heatmap && typeof stats.heatmap === 'object'
        ? stats.heatmap
        : {}) as Record<string, { total_attempts: number; correct: number; incorrect: number; avg_wpm_at_key: number }>;
      const blob = await renderShareCard(
        {
          wpm: stats.wpm,
          raw_wpm: stats.raw_wpm,
          accuracy: stats.accuracy,
          duration_ms: stats.duration_ms,
          mode: sessionModeType || 'test',
          language: sessionLanguage || '',
          date: new Date().toISOString().slice(0, 10),
          heatmap,
        },
        colors,
        {
          title: t(uiLang, 'share_card.symbols'),
          legend: t(uiLang, 'share_card.first_attempt'),
          empty: t(uiLang, 'share_card.empty'),
          space: t(uiLang, 'share_card.space'),
          shown: t(uiLang, 'share_card.shown'),
          remaining: t(uiLang, 'share_card.remaining'),
        },
      );
      shareState = 'saving';
      shareOutcome = await saveResultCard(Array.from(new Uint8Array(await blob.arrayBuffer())));
      shareState = 'done';
    } catch (e) {
      console.error('share png failed');
      shareState = 'error';
    }
  }
</script>

<div class="result-overlay">
  <h2>{t(uiLang, 'result.complete')}</h2>
  <div class="stats-grid">
    <div class="stat-box"><span class="stat-value">{stats.wpm.toFixed(1)}</span><span class="stat-label">WPM</span></div>
    <div class="stat-box"><span class="stat-value">{stats.raw_wpm.toFixed(1)}</span><span class="stat-label">{t(uiLang, 'result.raw_wpm')}</span></div>
    <div class="stat-box"><span class="stat-value">{stats.accuracy.toFixed(1)}%</span><span class="stat-label">{t(uiLang, 'result.accuracy')}</span></div>
    <div class="stat-box"><span class="stat-value">{stats.raw_accuracy.toFixed(1)}%</span><span class="stat-label">{t(uiLang, 'result.raw_acc')}</span></div>
  </div>
  <div class="stats-details">
    <span>{t(uiLang, 'result.correct')}: {stats.correct_chars}</span>
    <span>{t(uiLang, 'result.incorrect')}: {stats.incorrect_chars}</span>
    <span>{t(uiLang, 'result.backspaces')}: {stats.backspaces}</span>
    <span>{t(uiLang, 'result.duration')}: {(stats.duration_ms / 1000).toFixed(1)}s</span>
  </div>
  <KeyboardHeatmap heatmap={stats.heatmap} charStats={stats.char_stats} {keyboardLayout} />
  <div class="result-actions">
    {#if lessonNavigation}
      {#if lessonNavigation.nextLessonId}
        <button class="primary" onclick={onNextLesson}>{t(uiLang, 'result.next_lesson')}</button>
      {/if}
      <button onclick={onRepeatLesson}>{t(uiLang, 'result.repeat_lesson')}</button>
      <button onclick={onReturnToLessons}>{t(uiLang, 'result.back_to_lessons')}</button>
    {:else}
      <button onclick={onRestart}>{t(uiLang, 'result.restart')}</button>
    {/if}
    <button onclick={handleSharePng} disabled={shareState === 'rendering' || shareState === 'saving'}>
      {#if shareState === 'rendering' || shareState === 'saving'}<span class="share-spinner" aria-hidden="true"></span>{/if}
      {t(uiLang, 'result.share_png')}
    </button>
  </div>
  <div class="share-status" class:has-message={shareState !== 'idle'} role="status" aria-live="polite" aria-atomic="true">
    {#if shareState === 'rendering' || shareState === 'saving'}
      <p>{t(uiLang, shareState === 'rendering' ? 'result.png_rendering' : 'result.png_delivering')}</p>
      <div class="share-progress" aria-hidden="true"><span></span></div>
    {:else if shareOutcome}
      {#if shareOutcome.savedPath}
        <p class="share-confirmed">✓ {t(uiLang, 'result.png_saved')} <span class="share-path">{shareOutcome.savedPath}</span></p>
      {:else}<p class="share-error">{t(uiLang, 'result.png_save_failed')}</p>{/if}
      {#if shareOutcome.copied}
        <p class="share-confirmed">✓ {t(uiLang, 'result.png_copied')}</p>
      {:else}<p class="share-error">{t(uiLang, 'result.png_copy_failed')}</p>{/if}
    {:else if shareState === 'error'}<p class="share-error">{t(uiLang, 'result.share_png_error')}</p>{/if}
  </div>
</div>

<style>
  .result-overlay { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 1.5rem; }
  h2 { color: var(--main); font-size: 1.5rem; }
  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1.5rem; }
  .stat-box { display: flex; flex-direction: column; gap: 0.25rem; padding: 1.5rem 2rem; background-color: var(--bg-sub); border-radius: 8px; }
  .stat-value { font-size: 2rem; color: var(--main); }
  .stat-label { font-size: 0.75rem; color: var(--sub); text-transform: uppercase; }
  .stats-details { display: flex; gap: 2rem; font-size: 0.875rem; color: var(--sub); }
  .result-actions { display: flex; flex-wrap: wrap; justify-content: center; gap: .75rem; }
  .share-status { box-sizing: border-box; width: min(100%, 42rem); padding: 1rem 1.25rem; border: 1px solid var(--color-border, var(--sub)); border-left: 3px solid var(--main); border-radius: 10px; background: var(--bg-sub); text-align: left; font-size: .85rem; line-height: 1.5; color: var(--sub); }
  .share-status:not(.has-message) { display: none; }
  .share-status p { margin: 0; }
  .share-status p + p { margin-top: .75rem; }
  .share-confirmed { color: var(--main); animation: share-arrive .25s ease-out; }
  .share-path { display: block; margin-top: .4rem; padding: .55rem .7rem; border-radius: 5px; background: var(--bg); overflow-wrap: anywhere; user-select: text; font-size: .8rem; }
  .share-spinner { display: inline-block; width: .75em; height: .75em; margin-right: .5em; border: 2px solid currentColor; border-right-color: transparent; border-radius: 50%; animation: share-spin .8s linear infinite; }
  .share-progress { height: 3px; margin: .7rem auto 0; max-width: 16rem; overflow: hidden; background: var(--bg-sub); }
  .share-progress span { display: block; width: 35%; height: 100%; background: var(--main); animation: share-slide 1.2s ease-in-out infinite alternate; }
  @keyframes share-spin { to { transform: rotate(360deg); } }
  @keyframes share-slide { to { transform: translateX(185%); } }
  @keyframes share-arrive { from { opacity: 0; transform: translateY(3px); } to { opacity: 1; transform: translateY(0); } }
  @media (prefers-reduced-motion: reduce) { .share-spinner, .share-progress span, .share-confirmed { animation: none; } }
  .share-error { color: var(--error); font-size: 0.8rem; margin: 0; }
  button { background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main); padding: 0.5rem 2rem; font-family: inherit; font-size: 1rem; cursor: pointer; border-radius: 4px; }
  button.primary { background-color: var(--main); color: var(--color-accent-text); }
  button:hover { background-color: var(--main); color: var(--color-accent-text); }

  @media (max-width: 640px) {
    .stats-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); width: 100%; gap: .75rem; }
    .stat-box { padding: 1rem; }
    .stats-details { flex-wrap: wrap; justify-content: center; gap: .75rem 1rem; }
    .result-actions, .result-actions button { width: 100%; }
  }
</style>
