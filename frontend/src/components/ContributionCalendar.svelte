<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/api/ipc';
  import type { ProgressPoint } from '../lib/types/index';

  let points = $state<ProgressPoint[]>([]);
  let loading = $state(false);

  async function loadData() {
    loading = true;
    try {
      // 365 days of daily activity for the contribution grid.
      points = await ipc.getProgressHistory(365);
    } catch {
      points = [];
    }
    loading = false;
  }

  onMount(loadData);

  // Build a map date -> tests for O(1) lookup.
  let testsByDate = $derived(
    new Map(points.map((p) => [p.date, p.tests])),
  );

  // GitHub-style intensity buckets based on the max daily test count.
  let maxTests = $derived(Math.max(...points.map((p) => p.tests), 0));

  function level(tests: number): number {
    if (tests <= 0) return 0;
    if (maxTests <= 0) return 1;
    const ratio = tests / maxTests;
    if (ratio <= 0.25) return 1;
    if (ratio <= 0.5) return 2;
    if (ratio <= 0.75) return 3;
    return 4;
  }

  // Build the grid: weeks (columns) x days (rows), ending today.
  // GitHub shows ~53 weeks. Each cell is a day; empty future days are blank.
  const CELL = 18;
  const GAP = 4;
  const DAYS = 7;
  // Vertical space reserved for the month labels above the grid.
  const LABEL_H = 26;

  let grid = $derived.by(() => {
    const today = new Date();
    // Normalize to local midnight.
    const end = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    const start = new Date(end);
    start.setDate(end.getDate() - 364); // 365 days inclusive

    // Align start to Sunday so columns are full weeks.
    const startDow = start.getDay();
    start.setDate(start.getDate() - startDow);

    const cells: { date: string; tests: number; level: number; isFuture: boolean }[] = [];
    const cursor = new Date(start);
    while (cursor <= end) {
      const iso = cursor.toISOString().slice(0, 10);
      const tests = testsByDate.get(iso) ?? 0;
      cells.push({
        date: iso,
        tests,
        level: level(tests),
        isFuture: cursor > end,
      });
      cursor.setDate(cursor.getDate() + 1);
    }
    return cells;
  });

  const weeks = $derived(Math.ceil(grid.length / DAYS));
  const W = $derived(weeks * (CELL + GAP) + GAP);
  const H = $derived(LABEL_H + DAYS * (CELL + GAP) + GAP);

  function cellX(i: number): number {
    return GAP + Math.floor(i / DAYS) * (CELL + GAP);
  }
  function cellY(i: number): number {
    return LABEL_H + GAP + (i % DAYS) * (CELL + GAP);
  }

  // Month labels along the top.
  let monthLabels = $derived.by(() => {
    const labels: { x: number; text: string }[] = [];
    let lastMonth = -1;
    for (let i = 0; i < grid.length; i++) {
      const d = new Date(grid[i].date + 'T00:00:00');
      const m = d.getMonth();
      if (m !== lastMonth) {
        lastMonth = m;
        labels.push({ x: cellX(i), text: d.toLocaleString('en', { month: 'short' }) });
      }
    }
    return labels;
  });

  function formatTooltip(c: { date: string; tests: number }): string {
    const d = new Date(c.date + 'T00:00:00');
    const label = d.toLocaleDateString('en', { year: 'numeric', month: 'short', day: 'numeric' });
    if (c.tests <= 0) return `${label}: No activity`;
    return `${label}: ${c.tests} test${c.tests === 1 ? '' : 's'}`;
  }
</script>

<div class="contribution-calendar">
  <div class="cal-header">
    <h3>Activity</h3>
    <span class="cal-sub">Last 365 days</span>
  </div>

  {#if loading}
    <p class="empty">Loading...</p>
  {:else if grid.length === 0}
    <p class="empty">No activity yet. Complete tests to fill the calendar.</p>
  {:else}
    <div class="cal-scroll">
      <svg viewBox="0 0 {W} {H}" class="cal-svg" role="img" aria-label="Contribution calendar">
        {#each monthLabels as ml}
          <text x={ml.x} y="10" fill="var(--color-chart-label)" font-size="11">{ml.text}</text>
        {/each}
        {#each grid as c, i}
          <rect
            x={cellX(i)}
            y={cellY(i)}
            width={CELL}
            height={CELL}
            rx="2"
            class="cal-cell"
            class:lvl0={c.level === 0}
            class:lvl1={c.level === 1}
            class:lvl2={c.level === 2}
            class:lvl3={c.level === 3}
            class:lvl4={c.level === 4}
          >
            <title>{formatTooltip(c)}</title>
          </rect>
        {/each}
      </svg>
    </div>
    <div class="cal-legend">
      <span class="legend-label">Less</span>
      <span class="legend-cell lvl0"></span>
      <span class="legend-cell lvl1"></span>
      <span class="legend-cell lvl2"></span>
      <span class="legend-cell lvl3"></span>
      <span class="legend-cell lvl4"></span>
      <span class="legend-label">More</span>
    </div>
  {/if}
</div>

<style>
  .contribution-calendar { max-width: 1200px; width: 100%; margin-bottom: 2rem; }
  .cal-header { display: flex; align-items: baseline; gap: 0.75rem; margin-bottom: 0.5rem; }
  h3 { color: var(--main); font-size: 1.3rem; }
  .cal-sub { color: var(--sub); font-size: 0.85rem; }
  .cal-scroll { overflow-x: auto; }
  .cal-svg { display: block; min-width: 900px; }
  .cal-cell { stroke: none; }
  .lvl0 { fill: var(--color-chart-grid); }
  .lvl1 { fill: var(--color-chart-primary); opacity: 0.25; }
  .lvl2 { fill: var(--color-chart-primary); opacity: 0.5; }
  .lvl3 { fill: var(--color-chart-primary); opacity: 0.75; }
  .lvl4 { fill: var(--color-chart-primary); opacity: 1; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
  .cal-legend { display: flex; align-items: center; gap: 0.25rem; margin-top: 0.5rem; font-size: 0.8rem; color: var(--sub); }
  .legend-cell { width: 18px; height: 18px; border-radius: 3px; }
  .legend-label { margin: 0 0.25rem; }
</style>
