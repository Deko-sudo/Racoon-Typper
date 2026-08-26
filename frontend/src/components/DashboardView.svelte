<script lang="ts">
  import type { DashboardStatsResponse, WeeklySummaryResponse } from '../lib/types/index';
  import ContributionCalendar from './ContributionCalendar.svelte';
  import Icon from './Icon.svelte';
  import { t } from '../lib/i18n';

  let {
    stats,
    weekly = null,
    onNavigate,
    weakKeys = [],
    onStartTraining,
    uiLang = 'en',
  }: {
    stats: DashboardStatsResponse | null;
    weekly?: WeeklySummaryResponse[] | null;
    onNavigate: (v: string) => void;
    weakKeys?: { ch: string; accuracy: number; error_count: number }[];
    onStartTraining?: () => void;
    uiLang?: string;
  } = $props();

  // Top-3 weak keys for the training widget.
  let topWeakKeys = $derived(weakKeys.slice(0, 3));

  let weeklyMax = $derived(
    weekly && weekly.length > 0 ? Math.max(...weekly.map((w) => w.total_tests), 1) : 1,
  );

  function weekLabel(weekStartIso: string): string {
    return weekStartIso.slice(5).replace('-', '.');
  }

  function weekTitle(week: WeeklySummaryResponse): string {
    return `${week.week_start}: ${week.total_tests} ${t(uiLang, 'dash.weekly_tests')}`;
  }

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

    {#if weekly && weekly.length > 0}
      <div class="weekly-card">
        <span class="weekly-title">{t(uiLang, 'dash.weekly_title')}</span>
        <div class="weekly-strip">
          {#each weekly as week, index}
            <div class="week-col" class:current={index === weekly.length - 1} title={weekTitle(week)}>
              <span class="week-tests">{week.total_tests}</span>
              <div class="week-bar-track">
                <div
                  class="week-bar"
                  class:active={week.total_tests > 0}
                  style:height="{Math.round((week.total_tests / weeklyMax) * 100)}%"
                ></div>
              </div>
              <span class="week-goal" class:met={week.goal_met_days > 0}>
                {#if week.goal_met_days > 0}<Icon name="check" size="0.6rem" />{/if}
                {week.goal_met_days}
              </span>
              <span class="week-label">{weekLabel(week.week_start)}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

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

    <ContributionCalendar {uiLang} />
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
  .weekly-card {
    display: flex; flex-direction: column; gap: 0.75rem;
    padding: 1.25rem; margin-bottom: 2rem; background: var(--bg-sub); border-radius: 8px;
    border: 1px solid var(--sub);
  }
  .weekly-title { font-size: 0.875rem; font-weight: bold; color: var(--main); text-transform: uppercase; letter-spacing: 0.05em; }
  .weekly-strip { display: grid; grid-template-columns: repeat(auto-fit, minmax(2.4rem, 1fr)); gap: 0.5rem; }
  .week-col { display: flex; flex-direction: column; align-items: center; gap: 0.25rem; }
  .week-tests { font-size: 0.75rem; color: var(--text); }
  .week-bar-track {
    height: 3rem; width: 100%; max-width: 2rem;
    display: flex; align-items: flex-end;
    background: var(--bg); border-radius: 4px; overflow: hidden;
  }
  .week-bar { width: 100%; background: transparent; border-radius: 4px 4px 0 0; transition: height 0.2s; }
  .week-bar.active { background: var(--main); opacity: 0.55; }
  .week-col.current .week-bar.active { opacity: 1; }
  .week-goal {
    display: flex; align-items: center; gap: 0.15rem;
    font-size: 0.65rem; min-height: 0.85rem; color: var(--sub);
  }
  .week-goal.met { color: #6c8; }
  .week-label { font-size: 0.65rem; color: var(--sub); }
  .week-col.current .week-label { color: var(--main); }
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
