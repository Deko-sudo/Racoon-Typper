<script lang="ts">
  import type { PersonalBest } from '../types/index';

  let { bests }: { bests: PersonalBest[] } = $props();

  function formatDate(iso: string): string {
    try { return new Date(iso).toLocaleString(); } catch { return iso; }
  }
</script>

<div class="list-view">
  <h2>Personal Bests</h2>
  {#if bests.length === 0}
    <p class="empty">No records yet.</p>
  {:else}
    <table>
      <thead><tr><th>Mode</th><th>Best WPM</th><th>Best Accuracy</th><th>Updated</th></tr></thead>
      <tbody>
        {#each bests as b}
          <tr>
            <td>{b.mode_type}</td>
            <td>{b.best_wpm.toFixed(1)}</td>
            <td>{b.best_accuracy.toFixed(1)}%</td>
            <td>{formatDate(b.updated_at)}</td>
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