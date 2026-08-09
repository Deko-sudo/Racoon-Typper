<script lang="ts">
  import type { DashboardStatsResponse } from '../lib/types/index';
  import ProgressChart from './ProgressChart.svelte';
  import Icon from './Icon.svelte';
  import { t } from '../lib/i18n';

  let {
    stats,
    onNavigate,
    weakKeys = [],
    onStartTraining,
    uiLang = 'en',
  }: {
    stats: DashboardStatsResponse | null;
    onNavigate: (v: string) => void;
    weakKeys?: { ch: string; accuracy: number; error_count: number }[];
    onStartTraining?: () => void;
    uiLang?: string;
  } = $props();

  // Top-3 weak keys for the training widget.
  let topWeakKeys = $derived(weakKeys.slice(0, 3));

  function handleActionKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      event.stopPropagation();
      onNavigate('test');
    }
  }
</script>

<div class="dashboard">
  <h2>{t(uiLang, 'dash.title')}</h2>

  {#if stats}
    <div class="cards-grid">
      <div class="card streak-card">
        <span class="card-value">{stats.current_streak}</span>
        <span class="card-label">{t(uiLang, 'dash.current_streak')}</span>
        {#if stats.current_streak > 0}<span class="card-badge active"><Icon name="flame" size="0.75rem" /></span>{/if}
      </div>
      <div class="card">
        <span class="card-value">{stats.longest_streak}</span>
        <span class="card-label">{t(uiLang, 'dash.longest_streak')}</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.avg_wpm.toFixed(0)}</span>
        <span class="card-label">{t(uiLang, 'dash.avg_wpm')}</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.avg_accuracy.toFixed(1)}%</span>
        <span class="card-label">{t(uiLang, 'dash.avg_acc')}</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.tests_today}</span>
        <span class="card-label">{t(uiLang, 'dash.tests_today')}</span>
      </div>
      <div class="card">
        <span class="card-value">{stats.tests_this_week}</span>
        <span class="card-label">{t(uiLang, 'dash.tests_week')}</span>
      </div>
      <div class="card total-card">
        <span class="card-value">{stats.total_tests}</span>
        <span class="card-label">{t(uiLang, 'dash.total_tests')}</span>
      </div>
      <div class="card goal-card" class:goal-met={stats.daily_goal_met}>
        <span class="card-value goal-icon">
          {#if stats.daily_goal_met}<Icon name="check" size="1.8rem" />{:else}<Icon name="circle" size="1.8rem" />{/if}
        </span>
        <span class="card-label">{t(uiLang, 'dash.daily_goal')}</span>
      </div>
      <div class="card action-card" role="button" tabindex="0" onkeydown={handleActionKeydown} onclick={() => onNavigate('test')}>
        <span class="card-action">{t(uiLang, 'dash.start_test')}</span>
      </div>
    </div>

    {#if topWeakKeys.length > 0 && onStartTraining}
      <div class="training-card">
        <div class="training-header">
          <Icon name="flame" size="1.1rem" />
          <span class="training-title">{t(uiLang, 'dash.training_day')}</span>
        </div>
        <div class="training-keys">
          {#each topWeakKeys as wk}
            <span class="training-key" class:critical={wk.accuracy < 70}>
              <span class="key-char">{wk.ch}</span>
              <span class="key-acc">{wk.accuracy.toFixed(0)}%</span>
            </span>
          {/each}
        </div>
        <button class="training-btn" onclick={onStartTraining}>
          {t(uiLang, 'dash.start_training')}
        </button>
      </div>
    {/if}

    <ProgressChart />
  {:else}
    <p class="empty">{t(uiLang, 'dash.loading')}</p>
  {/if}
</div>

<style>
  .dashboard { max-width: 1200px; width: 100%; }
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
  .goal-card { border-color: var(--sub); }
  .goal-card.goal-met { border-color: #6c8; }
  .goal-card.goal-met .goal-icon { color: #6c8; }
  .action-card { cursor: pointer; justify-content: center; }
  .action-card:hover { border-color: var(--main); background: var(--bg); }
  .card-action { color: var(--main); font-size: 0.875rem; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
  .training-card {
    display: flex; flex-direction: column; align-items: center; gap: 0.75rem;
    padding: 1.25rem; margin-bottom: 2rem; background: var(--bg-sub); border-radius: 8px;
    border: 1px solid var(--main);
  }
  .training-header { display: flex; align-items: center; gap: 0.5rem; color: var(--main); }
  .training-title { font-size: 0.875rem; font-weight: bold; text-transform: uppercase; letter-spacing: 0.05em; }
  .training-keys { display: flex; gap: 1rem; }
  .training-key { display: flex; flex-direction: column; align-items: center; gap: 0.15rem; }
  .key-char { font-size: 1.5rem; font-weight: bold; color: var(--text); font-family: monospace; }
  .key-acc { font-size: 0.65rem; color: var(--sub); }
  .training-key.critical .key-acc { color: #ff6b35; }
  .training-btn {
    background: var(--main); color: var(--bg); border: none;
    padding: 0.5rem 1.5rem; font-family: inherit; font-size: 0.875rem;
    cursor: pointer; border-radius: 4px;
  }
  .training-btn:hover { opacity: 0.85; }
</style>
