<script lang="ts">
  import type { TestSummary } from '../lib/types/index';
  import { t } from '../lib/i18n';

  let { history, total, uiLang = 'en' }: { history: TestSummary[]; total: number; uiLang?: string } = $props();

  function formatDate(iso: string): string {
    try { return new Date(iso).toLocaleString(); } catch { return iso; }
  }
</script>

<div class="list-view">
  <h2>{t(uiLang, 'history.title')} ({total})</h2>
  {#if history.length === 0}
    <p class="empty">{t(uiLang, 'history.empty')}</p>
  {:else}
    <table>
      <thead><tr><th>{t(uiLang, 'history.date')}</th><th>{t(uiLang, 'history.mode')}</th><th>{t(uiLang, 'history.wpm')}</th><th>{t(uiLang, 'history.accuracy')}</th><th>{t(uiLang, 'history.duration')}</th><th>{t(uiLang, 'history.pb')}</th></tr></thead>
      <tbody>
        {#each history as h}
          <tr>
            <td>{formatDate(h.created_at)}</td>
            <td>{h.mode_type}</td>
            <td>{h.wpm.toFixed(1)}</td>
            <td>{h.accuracy.toFixed(1)}%</td>
            <td>{(h.duration_ms / 1000).toFixed(1)}s</td>
            <td>{h.is_pb ? '★' : ''}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .list-view { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
  table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
  th { text-align: left; color: var(--main); padding: 0.5rem; border-bottom: 1px solid var(--bg-sub); }
  td { padding: 0.5rem; color: var(--text); border-bottom: 1px solid var(--bg-sub); }
</style>