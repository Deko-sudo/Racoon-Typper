<script lang="ts">
  import type { FinalStats } from '../types/index';
  import KeyboardHeatmap from './KeyboardHeatmap.svelte';

  let { stats, onRestart }: { stats: FinalStats; onRestart: () => void } = $props();
</script>

<div class="result-overlay">
  <h2>Test Complete</h2>
  <div class="stats-grid">
    <div class="stat-box"><span class="stat-value">{stats.wpm.toFixed(1)}</span><span class="stat-label">WPM</span></div>
    <div class="stat-box"><span class="stat-value">{stats.raw_wpm.toFixed(1)}</span><span class="stat-label">Raw WPM</span></div>
    <div class="stat-box"><span class="stat-value">{stats.accuracy.toFixed(1)}%</span><span class="stat-label">Accuracy</span></div>
    <div class="stat-box"><span class="stat-value">{stats.raw_accuracy.toFixed(1)}%</span><span class="stat-label">Raw Acc</span></div>
  </div>
  <div class="stats-details">
    <span>Correct: {stats.correct_chars}</span>
    <span>Incorrect: {stats.incorrect_chars}</span>
    <span>Backspaces: {stats.backspaces}</span>
    <span>Duration: {(stats.duration_ms / 1000).toFixed(1)}s</span>
  </div>
  <KeyboardHeatmap heatmap={stats.heatmap} />
  <button onclick={onRestart}>Restart</button>
</div>

<style>
  .result-overlay { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 1.5rem; }
  h2 { color: var(--main); font-size: 1.5rem; }
  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1.5rem; }
  .stat-box { display: flex; flex-direction: column; gap: 0.25rem; padding: 1.5rem 2rem; background-color: var(--bg-sub); border-radius: 8px; }
  .stat-value { font-size: 2rem; color: var(--main); }
  .stat-label { font-size: 0.75rem; color: var(--sub); text-transform: uppercase; }
  .stats-details { display: flex; gap: 2rem; font-size: 0.875rem; color: var(--sub); }
  button { background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main); padding: 0.5rem 2rem; font-family: inherit; font-size: 1rem; cursor: pointer; border-radius: 4px; }
  button:hover { background-color: var(--main); color: var(--bg); }
</style>