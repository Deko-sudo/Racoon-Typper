<script lang="ts">
  import type { DashboardStatsResponse } from '../types/index';
  import ProgressChart from './ProgressChart.svelte';

  let { stats, onNavigate }: { stats: DashboardStatsResponse | null; onNavigate: (v: string) => void } = $props();
</script>

<div class="dashboard">
  <h2>Dashboard</h2>

  {#if stats}
    <div class="cards-grid">
      <div class="card streak-card">
        <span class="card-value">{stats.current_streak}</span>
        <span class="card-label">Current Streak</span>
        {#if stats.current_streak > 0}<span class="card-badge active">🔥 Active</span>{/if}
      </div>
      <div class="card">
        <span class="card-value">{stats.longest_streak}</span>
        <span class="card-label">Longest Streak</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.avg_wpm.toFixed(0)}</span>
        <span class="card-label">Avg WPM (7d)</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.avg_accuracy.toFixed(1)}%</span>
        <span class="card-label">Avg Accuracy (7d)</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.tests_today}</span>
        <span class="card-label">Tests Today</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.tests_this_week}</span>
        <span class="card-label">Tests This Week</span>
      </div>
      <div class="card total-card">
        <span class="card-value">{stats.total_tests}</span>
        <span class="card-label">Total Tests</span>
      </div>
      <div class="card action-card" role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && onNavigate('test')} onclick={() => onNavigate('test')}>
        <span class="card-action">Start Test →</span>
      </div>
    </div>

    <ProgressChart />
  {:else}
    <p class="empty">Loading dashboard...</p>
  {/if}
</div>

<style>
  .dashboard { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  .cards-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 0.75rem; margin-bottom: 2rem; }
  .card {
    display: flex; flex-direction: column; align-items: center; gap: 0.25rem;
    padding: 1.25rem 1rem; background: var(--bg-sub); border-radius: 8px;
    border: 1px solid transparent; transition: border-color 0.2s;
  }
  .card:hover { border-color: var(--sub); }
  .streak-card { border-color: var(--main); }
  .card-value { font-size: 2rem; color: var(--main); font-weight: bold; }
  .card-label { font-size: 0.7rem; color: var(--sub); text-transform: uppercase; }
  .card-badge { font-size: 0.65rem; color: var(--main); }
  .card-badge.active { color: #ff6b35; }
  .total-card { border-color: var(--sub); }
  .action-card { cursor: pointer; justify-content: center; }
  .action-card:hover { border-color: var(--main); background: var(--bg); }
  .card-action { color: var(--main); font-size: 0.875rem; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
</style>