<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/api/ipc';
  import { t } from '../lib/i18n';

  let { uiLang = 'en' }: { uiLang?: string } = $props();

  let achievements = $state<Array<{ id: string; name: string; description: string; unlocked: boolean }>>([]);
  let insights = $state<Array<{ level: string; title: string; message: string }>>([]);
  let consistency = $state<{ score: number; mean_wpm: number; std_dev: number; cv: number; samples: number } | null>(null);
  let exportFormat = $state<'json' | 'csv'>('json');
  let exportResult = $state('');
  let errorMsg = $state('');

  async function loadData() {
    try {
      const ach = await ipc.getAchievements() as Array<{ id: string; name: string; description: string; unlocked: boolean }>[];
      achievements = ach[0] || [];
      const ins = await ipc.getInsights() as Array<{ level: string; title: string; message: string }>[];
      insights = ins[0] || [];
      consistency = await ipc.getConsistency() as any;
    } catch (e) {
      errorMsg = `Error: ${e}`;
    }
  }

  async function doExport() {
    try {
      exportResult = await ipc.exportData(exportFormat);
    } catch (e) {
      errorMsg = `Export error: ${e}`;
    }
  }

  function downloadExport() {
    if (!exportResult) return;
    const blob = new Blob([exportResult], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `racoon-typper-export.${exportFormat}`;
    a.click();
    URL.revokeObjectURL(url);
  }

  onMount(loadData);
</script>

<div class="analytics-view">
  <h2>{t(uiLang, 'analytics.title')}</h2>

  {#if errorMsg}<p class="error">{errorMsg}</p>{/if}

  {#if consistency}
    <div class="consistency-card">
      <h3>{t(uiLang, 'analytics.consistency')}</h3>
      <div class="consistency-score" style="color: {consistency.score > 80 ? 'var(--main)' : consistency.score > 60 ? 'var(--text)' : 'var(--error)'}">
        {consistency.score.toFixed(0)}%
      </div>
      <div class="consistency-details">
        <span>{t(uiLang, 'analytics.mean_wpm')}: {consistency.mean_wpm.toFixed(1)}</span>
        <span>{t(uiLang, 'analytics.std_dev')}: {consistency.std_dev.toFixed(1)}</span>
        <span>CV: {(consistency.cv * 100).toFixed(1)}%</span>
        <span>{t(uiLang, 'analytics.samples')}: {consistency.samples}</span>
      </div>
    </div>
  {:else}
    <p class="empty">{t(uiLang, 'analytics.empty_consistency')}</p>
  {/if}

  {#if insights.length > 0}
    <div class="insights-section">
      <h3>{t(uiLang, 'analytics.insights')}</h3>
      {#each insights as ins}
        <div class="insight-card {ins.level}">
          <strong>{ins.title}</strong>
          <p>{ins.message}</p>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty">{t(uiLang, 'analytics.empty_insights')}</p>
  {/if}

  {#if achievements.length > 0}
    <div class="achievements-section">
      <h3>{t(uiLang, 'analytics.achievements')}</h3>
      <div class="achievements-grid">
        {#each achievements as ach}
          <div class="achievement-card" class:locked={!ach.unlocked}>
            <span class="ach-icon">{ach.unlocked ? '🏆' : '🔒'}</span>
            <span class="ach-name">{ach.name}</span>
            <span class="ach-desc">{ach.description}</span>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <p class="empty">{t(uiLang, 'analytics.empty_achievements')}</p>
  {/if}

  <div class="export-section">
    <h3>{t(uiLang, 'analytics.export')}</h3>
    <div class="export-controls">
      <select bind:value={exportFormat}>
        <option value="json">JSON</option>
        <option value="csv">CSV</option>
      </select>
      <button onclick={doExport}>{t(uiLang, 'analytics.generate')}</button>
      {#if exportResult}<button onclick={downloadExport}>{t(uiLang, 'analytics.download')}</button>{/if}
    </div>
    {#if exportResult}
      <pre class="export-preview">{exportResult.substring(0, 500)}{exportResult.length > 500 ? '...' : ''}</pre>
    {/if}
  </div>
</div>

<style>
  .analytics-view { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 1rem 0 0.5rem; }
  .consistency-card { display: flex; flex-direction: column; align-items: center; gap: 0.5rem; padding: 1.5rem; background: var(--bg-sub); border-radius: 8px; margin-bottom: 1rem; }
  .consistency-score { font-size: 3rem; font-weight: bold; }
  .consistency-details { display: flex; gap: 1.5rem; font-size: 0.75rem; color: var(--sub); }
  .insights-section { margin-bottom: 1.5rem; }
  .insight-card { padding: 0.75rem 1rem; border-radius: 6px; margin-bottom: 0.5rem; }
  .insight-card.success { background: rgba(100,200,100,0.1); border-left: 3px solid #6c8; }
  .insight-card.warning { background: rgba(226,183,20,0.1); border-left: 3px solid var(--main); }
  .insight-card.info { background: rgba(85,85,85,0.1); border-left: 3px solid var(--sub); }
  .insight-card strong { color: var(--text); display: block; font-size: 0.875rem; }
  .insight-card p { color: var(--sub); font-size: 0.75rem; margin: 0.25rem 0 0; }
  .achievements-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 0.5rem; }
  .achievement-card { display: flex; flex-direction: column; align-items: center; gap: 0.25rem; padding: 1rem; background: var(--bg-sub); border-radius: 6px; border: 1px solid transparent; }
  .achievement-card.locked { opacity: 0.4; }
  .ach-icon { font-size: 1.5rem; }
  .ach-name { font-size: 0.75rem; color: var(--main); font-weight: bold; text-align: center; }
  .ach-desc { font-size: 0.65rem; color: var(--sub); text-align: center; }
  .export-section { margin-top: 1.5rem; }
  .export-controls { display: flex; gap: 0.5rem; align-items: center; margin-bottom: 0.5rem; }
  .export-controls select, .export-controls button {
    background: var(--bg-sub); border: 1px solid var(--sub); color: var(--text);
    padding: 0.5rem 1rem; font-family: inherit; font-size: 0.75rem; border-radius: 4px; cursor: pointer;
  }
  .export-preview { background: var(--bg-sub); padding: 0.75rem; border-radius: 4px; font-size: 0.65rem; overflow-x: auto; max-height: 200px; }
  .error { color: var(--error); font-size: 0.875rem; }
</style>