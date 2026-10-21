<script lang="ts">
  import { t } from '../lib/i18n';
  import type { AppSettings } from '../lib/types/index';

  let {
    settings,
    uiLang = 'en',
    onUpdateSetting,
    onPhaseComplete,
  }: {
    settings: AppSettings | null;
    uiLang?: string;
    onUpdateSetting: (key: string, value: unknown) => void;
    onPhaseComplete: () => void;
  } = $props();

  const LONG_BREAK_MIN = 15;
  const CYCLES_BEFORE_LONG_BREAK = 4;

  type Phase = 'work' | 'break' | 'long_break';

  let phase = $state<Phase>('work');
  let running = $state(false);
  let remainingMs = $state(0);
  let completedCycles = $state(0);
  let phaseEndedAt = $state<number | null>(null);

  let workMin = $derived(settings?.pomodoro_work_min ?? 25);
  let breakMin = $derived(settings?.pomodoro_break_min ?? 5);

  function phaseDurationMs(nextPhase: Phase): number {
    const minutes = nextPhase === 'work'
      ? workMin
      : nextPhase === 'break'
        ? breakMin
        : LONG_BREAK_MIN;
    return minutes * 60_000;
  }

  function startPhase(nextPhase: Phase) {
    phase = nextPhase;
    remainingMs = phaseDurationMs(nextPhase);
    phaseEndedAt = Date.now() + remainingMs;
    running = true;
  }

  function start() {
    startPhase(phase);
  }

  function pause() {
    if (!running) return;
    running = false;
    remainingMs = Math.max(0, (phaseEndedAt ?? Date.now()) - Date.now());
    phaseEndedAt = null;
  }

  function resume() {
    if (running || remainingMs <= 0) return;
    phaseEndedAt = Date.now() + remainingMs;
    running = true;
  }

  function reset() {
    running = false;
    phase = 'work';
    completedCycles = 0;
    remainingMs = phaseDurationMs('work');
    phaseEndedAt = null;
  }

  function advancePhase() {
    if (phase === 'work') {
      completedCycles += 1;
      const next: Phase = completedCycles % CYCLES_BEFORE_LONG_BREAK === 0
        ? 'long_break'
        : 'break';
      startPhase(next);
    } else {
      startPhase('work');
    }
    onPhaseComplete();
  }

  // Drift-free timer: Date.now()-delta survives background-tab throttling
  // (same pattern as the test clock in App.svelte).
  $effect(() => {
    if (!running || phaseEndedAt === null) return;
    const endAt = phaseEndedAt;
    const updateClock = () => {
      const left = Math.max(0, endAt - Date.now());
      remainingMs = left;
      if (left <= 0) {
        advancePhase();
      }
    };
    updateClock();
    const interval = window.setInterval(updateClock, 200);
    return () => window.clearInterval(interval);
  });

  function formatTime(ms: number): string {
    const totalSeconds = Math.ceil(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
  }

  function phaseLabel(): string {
    if (phase === 'work') return t(uiLang, 'pomodoro.work');
    if (phase === 'break') return t(uiLang, 'pomodoro.break');
    return t(uiLang, 'pomodoro.long_break');
  }

  function updateWorkMin(value: number) {
    if (!Number.isFinite(value)) return;
    onUpdateSetting('pomodoro_work_min', Math.min(180, Math.max(1, Math.round(value))));
    if (!running && phase === 'work') remainingMs = phaseDurationMs('work');
  }

  function updateBreakMin(value: number) {
    if (!Number.isFinite(value)) return;
    onUpdateSetting('pomodoro_break_min', Math.min(180, Math.max(1, Math.round(value))));
    if (!running && phase === 'break') remainingMs = phaseDurationMs('break');
  }
</script>

<div class="pomodoro-view">
  <h2>{t(uiLang, 'pomodoro.title')}</h2>

  <div class="pomodoro-card" class:work={phase === 'work'} class:break={phase !== 'work'}>
    <div class="phase-label">{phaseLabel()}</div>
    <div class="timer">{formatTime(remainingMs)}</div>
    <div class="cycle-info">
      {t(uiLang, 'pomodoro.cycle')} {completedCycles % CYCLES_BEFORE_LONG_BREAK + 1}/{CYCLES_BEFORE_LONG_BREAK}
      · {t(uiLang, 'pomodoro.cycles')}: {completedCycles}
    </div>
    <div class="pomodoro-actions">
      {#if running}
        <button class="primary" onclick={pause}>{t(uiLang, 'pomodoro.pause')}</button>
      {:else if remainingMs > 0 && remainingMs < phaseDurationMs(phase)}
        <button class="primary" onclick={resume}>{t(uiLang, 'pomodoro.resume')}</button>
      {:else}
        <button class="primary" onclick={start}>{t(uiLang, 'pomodoro.start')}</button>
      {/if}
      <button onclick={reset}>{t(uiLang, 'pomodoro.reset')}</button>
    </div>
  </div>

  <div class="pomodoro-settings">
    <div class="setting-row">
      <label for="pomodoro-work-min">{t(uiLang, 'pomodoro.work_min')}</label>
      <input
        id="pomodoro-work-min"
        type="number"
        min="1"
        max="180"
        value={workMin}
        onchange={(e) => updateWorkMin(Number(e.currentTarget.value))}
      />
    </div>
    <div class="setting-row">
      <label for="pomodoro-break-min">{t(uiLang, 'pomodoro.break_min')}</label>
      <input
        id="pomodoro-break-min"
        type="number"
        min="1"
        max="180"
        value={breakMin}
        onchange={(e) => updateBreakMin(Number(e.currentTarget.value))}
      />
    </div>
  </div>
</div>

<style>
  .pomodoro-view { max-width: 560px; width: 100%; display: flex; flex-direction: column; align-items: center; gap: 1.5rem; }
  h2 { color: var(--main); font-size: 1.5rem; }
  .pomodoro-card {
    width: 100%; display: flex; flex-direction: column; align-items: center; gap: 0.75rem;
    padding: 2.5rem 2rem; border: 1px solid var(--color-border); border-radius: 12px;
    background: var(--color-surface-raised); box-shadow: var(--shadow-surface);
  }
  .pomodoro-card.work { border-color: var(--color-chart-positive); }
  .pomodoro-card.break { border-color: var(--color-warning); }
  .phase-label { color: var(--main); text-transform: uppercase; letter-spacing: 0.08em; font-size: 0.7rem; font-weight: 700; }
  .timer { font-size: 4.5rem; font-weight: 700; color: var(--text); font-variant-numeric: tabular-nums; }
  .cycle-info { color: var(--sub); font-size: 0.75rem; }
  .pomodoro-actions { display: flex; gap: 0.75rem; }
  .pomodoro-actions button {
    background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main);
    padding: 0.5rem 1.5rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px;
  }
  .pomodoro-actions button.primary { background-color: var(--main); color: var(--color-accent-text); }
  .pomodoro-actions button:hover { opacity: 0.85; }
  .pomodoro-settings { width: 100%; display: flex; flex-direction: column; gap: 0.75rem; }
  .setting-row { display: flex; align-items: center; gap: 1rem; }
  .setting-row label { min-width: 180px; color: var(--sub); font-size: 0.875rem; }
  .setting-row input {
    background-color: var(--bg-sub); border: 1px solid var(--sub); color: var(--text);
    padding: 0.5rem; font-family: inherit; border-radius: 4px; font-size: 0.875rem; width: 90px;
  }
  .setting-row input:focus-visible { outline: 2px solid var(--color-focus-ring); outline-offset: 2px; }
</style>
