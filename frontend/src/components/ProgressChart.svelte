<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/api/ipc';
  import type { ProgressPoint } from '../types/index';

  let points = $state<ProgressPoint[]>([]);
  let period = $state<7 | 30 | 90>(7);
  let loading = $state(false);

  async function loadData() {
    loading = true;
    try {
      points = await ipc.getProgressHistory(period);
    } catch {
      points = [];
    }
    loading = false;
  }

  onMount(loadData);

  // SVG chart dimensions
  const W = 800;
  const H = 200;
  const PADDING = 40;

  let maxWpm = $derived(Math.max(...points.map(p => p.wpm), 1));
  let minWpm = $derived(0);
  let maxAcc = $derived(100);
  let minAcc = $derived(Math.min(...points.map(p => p.accuracy), 100) - 5);

  // Map points to SVG coordinates
  function xCoord(i: number): number {
    if (points.length <= 1) return PADDING;
    return PADDING + (i / (points.length - 1)) * (W - 2 * PADDING);
  }

  function yWpm(wpm: number): number {
    let range = maxWpm - minWpm;
    if (range <= 0) range = 1;
    return H - PADDING - ((wpm - minWpm) / range) * (H - 2 * PADDING);
  }

  function yAcc(acc: number): number {
    let range = maxAcc - minAcc;
    if (range <= 0) range = 1;
    return H - PADDING - ((acc - minAcc) / range) * (H - 2 * PADDING);
  }

  let wpmPath = $derived.by(() => {
    if (points.length === 0) return '';
    return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${xCoord(i)} ${yWpm(p.wpm)}`).join(' ');
  });

  let accPath = $derived.by(() => {
    if (points.length === 0) return '';
    return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${xCoord(i)} ${yAcc(p.accuracy)}`).join(' ');
  });

  function selectPeriod(p: 7 | 30 | 90) {
    period = p;
    loadData();
  }

  function formatDate(d: string): string {
    return d.substring(5);
  }
</script>

<div class="progress-chart">
  <div class="chart-header">
    <h3>Progress</h3>
    <div class="period-selector">
      <button class:active={period === 7} onclick={() => selectPeriod(7)}>7d</button>
      <button class:active={period === 30} onclick={() => selectPeriod(30)}>30d</button>
      <button class:active={period === 90} onclick={() => selectPeriod(90)}>90d</button>
    </div>
  </div>

  {#if loading}
    <p class="empty">Loading...</p>
  {:else if points.length === 0}
    <p class="empty">No data yet. Complete tests to see progress.</p>
  {:else}
    <svg viewBox="0 0 {W} {H}" class="chart-svg">
      <!-- Grid lines -->
      <line x1="{PADDING}" y1="{H - PADDING}" x2="{W - PADDING}" y2="{H - PADDING}" stroke="var(--sub)" stroke-width="0.5" />
      <line x1="{PADDING}" y1="{PADDING}" x2="{PADDING}" y2="{H - PADDING}" stroke="var(--sub)" stroke-width="0.5" />

      <!-- WPM line -->
      <path d={wpmPath} fill="none" stroke="var(--main)" stroke-width="2" />
      <!-- Accuracy line -->
      <path d={accPath} fill="none" stroke="var(--text)" stroke-width="1.5" stroke-dasharray="4 2" />

      <!-- Labels -->
      <text x="{PADDING - 5}" y="{PADDING + 5}" fill="var(--main)" font-size="10" text-anchor="end">WPM</text>
      <text x="{PADDING - 5}" y="{PADDING + 20}" fill="var(--text)" font-size="10" text-anchor="end">Acc</text>

      <!-- X-axis labels -->
      {#each points as p, i}
        {#if i % Math.max(1, Math.floor(points.length / 6)) === 0}
          <text x={xCoord(i)} y="{H - PADDING + 15}" fill="var(--sub)" font-size="9" text-anchor="middle">{formatDate(p.date)}</text>
        {/if}
      {/each}

      <!-- Data points -->
      {#each points as p, i}
        <circle cx={xCoord(i)} cy={yWpm(p.wpm)} r="2.5" fill="var(--main)" />
      {/each}
    </svg>
    <div class="chart-legend">
      <span class="legend-item"><span class="legend-dot wpm"></span> WPM</span>
      <span class="legend-item"><span class="legend-dot acc"></span> Accuracy</span>
      <span class="legend-info">{points.length} data points</span>
    </div>
  {/if}
</div>

<style>
  .progress-chart { max-width: 900px; width: 100%; }
  .chart-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem; }
  h3 { color: var(--main); font-size: 1.1rem; }
  .period-selector { display: flex; gap: 0.25rem; }
  .period-selector button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.2rem 0.6rem; font-family: inherit; font-size: 0.7rem; cursor: pointer; border-radius: 4px;
  }
  .period-selector button.active { color: var(--main); border-color: var(--main); }
  .chart-svg { width: 100%; height: auto; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
  .chart-legend { display: flex; gap: 1.5rem; align-items: center; margin-top: 0.5rem; font-size: 0.75rem; }
  .legend-item { display: flex; gap: 0.25rem; align-items: center; color: var(--text); }
  .legend-dot { width: 12px; height: 2px; border-radius: 1px; }
  .legend-dot.wpm { background: var(--main); }
  .legend-dot.acc { background: var(--text); border-top: 1px dashed var(--text); }
  .legend-info { color: var(--sub); margin-left: auto; }
</style>