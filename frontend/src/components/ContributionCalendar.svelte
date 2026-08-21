<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/api/ipc';
  import { t } from '../lib/i18n';
  import type { ProgressPoint } from '../lib/types/index';
  import {
    buildGrid,
    formatTooltip,
    legendRanges,
    type CalendarMetric,
    type TooltipLabels,
  } from '../lib/contributionCalendar';

  let {
    uiLang = 'en',
  }: {
    uiLang?: string;
  } = $props();

  let points = $state<ProgressPoint[]>([]);
  let loading = $state(false);
  let metric = $state<CalendarMetric>('tests');

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

  // Build the grid: weeks (columns) x days (rows), ending today.
  // GitHub shows ~53 weeks. Each cell is a day; empty future days are blank.
  const CELL = 18;
  const GAP = 4;
  const DAYS = 7;
  // Vertical space reserved for the month labels above the grid.
  const LABEL_H = 26;

  let grid = $derived(buildGrid(points, new Date(), metric));

  const weeks = $derived(Math.ceil(grid.length / DAYS));
  const W = $derived(weeks * (CELL + GAP) + GAP);
  const H = $derived(LABEL_H + DAYS * (CELL + GAP) + GAP);

  function cellX(i: number): number {
    return GAP + Math.floor(i / DAYS) * (CELL + GAP);
  }
  function cellY(i: number): number {
    return LABEL_H + GAP + (i % DAYS) * (CELL + GAP);
  }

  // Month labels along the top, localized to the UI language.
  let monthLabels = $derived.by(() => {
    const labels: { x: number; text: string }[] = [];
    let lastMonth = -1;
    for (let i = 0; i < grid.length; i++) {
      const d = new Date(grid[i].date + 'T00:00:00');
      const m = d.getMonth();
      if (m !== lastMonth) {
        lastMonth = m;
        labels.push({ x: cellX(i), text: d.toLocaleString(uiLang, { month: 'short' }) });
      }
    }
    return labels;
  });

  // Numeric legend ranges for the current metric's max daily value.
  let legend = $derived.by(() => {
    const maxValue = Math.max(...grid.map((c) => c.value), 0);
    return legendRanges(maxValue);
  });

  let tooltipLabels = $derived<TooltipLabels>({
    noActivity: t(uiLang, 'calendar.no_activity'),
    minutes: t(uiLang, 'calendar.minutes'),
    lesson: {
      one: t(uiLang, 'calendar.lesson_one'),
      few: t(uiLang, 'calendar.lesson_few'),
      many: t(uiLang, 'calendar.lesson_many'),
      other: t(uiLang, 'calendar.lesson_other'),
    },
    test: {
      one: t(uiLang, 'calendar.test_one'),
      few: t(uiLang, 'calendar.test_few'),
      many: t(uiLang, 'calendar.test_many'),
      other: t(uiLang, 'calendar.test_other'),
    },
  });

  function tooltipFor(c: { date: string; value: number }): string {
    return formatTooltip(c, metric, uiLang, tooltipLabels);
  }

  function legendLabel(range: [number, number] | null): string {
    if (!range) return '—';
    if (range[0] === range[1]) return String(range[0]);
    return `${range[0]}–${range[1]}`;
  }
</script>

<div class="contribution-calendar">
  <div class="cal-header">
    <h3>{t(uiLang, 'calendar.title')}</h3>
    <span class="cal-sub">{t(uiLang, 'calendar.subtitle')}</span>
    <div class="metric-selector" role="group" aria-label={t(uiLang, 'calendar.metric_label')}>
      <button class:active={metric === 'tests'} onclick={() => (metric = 'tests')}>{t(uiLang, 'calendar.metric_tests')}</button>
      <button class:active={metric === 'time'} onclick={() => (metric = 'time')}>{t(uiLang, 'calendar.metric_time')}</button>
      <button class:active={metric === 'lessons'} onclick={() => (metric = 'lessons')}>{t(uiLang, 'calendar.metric_lessons')}</button>
    </div>
  </div>

  {#if loading}
    <p class="empty">{t(uiLang, 'calendar.loading')}</p>
  {:else if grid.length === 0}
    <p class="empty">{t(uiLang, 'calendar.empty')}</p>
  {:else}
    <div class="cal-scroll">
      <svg viewBox="0 0 {W} {H}" class="cal-svg" role="img" aria-label={t(uiLang, 'calendar.aria_label')}>
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
            class:today={c.isToday}
          >
            <title>{tooltipFor(c)}</title>
          </rect>
        {/each}
      </svg>
    </div>
    <div class="cal-legend">
      <span class="legend-label">{t(uiLang, 'calendar.less')}</span>
      {#each legend as range, i}
        <span class="legend-item">
          <span class="legend-cell lvl{i}"></span>
          <span class="legend-range">{legendLabel(range)}</span>
        </span>
      {/each}
      <span class="legend-label">{t(uiLang, 'calendar.more')}</span>
    </div>
  {/if}
</div>

<style>
  .contribution-calendar { max-width: 1200px; width: 100%; margin-bottom: 2rem; }
  .cal-header { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; flex-wrap: wrap; }
  h3 { color: var(--main); font-size: 1.3rem; }
  .cal-sub { color: var(--sub); font-size: 0.85rem; }
  .metric-selector { display: flex; gap: 0.25rem; margin-left: auto; }
  .metric-selector button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.2rem 0.6rem; font-family: inherit; font-size: 0.7rem; cursor: pointer; border-radius: 4px;
  }
  .metric-selector button.active { color: var(--main); border-color: var(--main); }
  .cal-scroll { overflow-x: auto; }
  .cal-svg { display: block; min-width: 900px; }
  .cal-cell { stroke: none; }
  .lvl0 { fill: var(--color-chart-grid); }
  .lvl1 { fill: var(--color-chart-primary); opacity: 0.25; }
  .lvl2 { fill: var(--color-chart-primary); opacity: 0.5; }
  .lvl3 { fill: var(--color-chart-primary); opacity: 0.75; }
  .lvl4 { fill: var(--color-chart-primary); opacity: 1; }
  .cal-cell.today { stroke: var(--color-caret); stroke-width: 1.5; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
  .cal-legend { display: flex; align-items: center; gap: 0.25rem; margin-top: 0.5rem; font-size: 0.8rem; color: var(--sub); }
  .legend-cell { width: 18px; height: 18px; border-radius: 3px; }
  .legend-label { margin: 0 0.25rem; }
  .legend-item { display: flex; align-items: center; gap: 0.2rem; }
  .legend-range { font-size: 0.65rem; color: var(--sub); min-width: 1.6em; text-align: center; }
</style>
