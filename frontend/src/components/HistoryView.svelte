<script lang="ts">
  import type { TestSummary } from '../types/index';

  let { history, total }: { history: TestSummary[]; total: number } = $props();

  function formatDate(iso: string): string {
    try { return new Date(iso).toLocaleString(); } catch { return iso; }
  }
</script>

<div class="list-view">
  <h2>Test History ({total})</h2>
  {#if history.length === 0}
    <p class="empty">No tests yet.</p>
  {:else}
    <table>
      <thead><tr><th>Date</th><th>Mode</th><th>WPM</th><th>Accuracy</th><th>Duration</th><th>PB</th></tr></thead>
      <tbody>
        {#each history as t}
          <tr>
            <td>{formatDate(t.created_at)}</td>
            <td>{t.mode_type}</td>
            <td>{t.wpm.toFixed(1)}</td>
            <td>{t.accuracy.toFixed(1)}%</td>
            <td>{(t.duration_ms / 1000).toFixed(1)}s</td>
            <td>{t.is_pb ? '★' : ''}</td>
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