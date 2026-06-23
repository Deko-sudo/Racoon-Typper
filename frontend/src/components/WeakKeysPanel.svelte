<script lang="ts">
  import KeyboardVizComponent from './KeyboardVizComponent.svelte';

  let {
    weakKeys = [],
    charStats = {},
    onGenerateTraining,
  }: {
    weakKeys: Array<{ ch: string; error_count: number; accuracy: number; rank: number }>;
    charStats: Record<string, { correct: number; incorrect: number; total: number }>;
    onGenerateTraining: () => void;
  } = $props();
</script>

<div class="weak-keys-panel">
  <h3>Weak Keys Analysis</h3>

  {#if weakKeys.length === 0}
    <p class="empty">No weak keys detected. Complete a test to see analysis.</p>
  {:else}
    <div class="weak-keys-list">
      {#each weakKeys as wk}
        <div class="weak-key-row" class:critical={wk.accuracy < 70}>
          <span class="wk-char">{wk.ch}</span>
          <span class="wk-accuracy">{wk.accuracy.toFixed(1)}%</span>
          <span class="wk-errors">{wk.error_count} errors</span>
          <span class="wk-rank">#{wk.rank}</span>
        </div>
      {/each}
    </div>
    <button onclick={onGenerateTraining}>Generate Training Text</button>
  {/if}

  <KeyboardVizComponent {charStats} />
</div>

<style>
  .weak-keys-panel { max-width: 700px; width: 100%; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; }
  .empty { color: var(--sub); text-align: center; padding: 1rem; }
  .weak-keys-list { display: flex; flex-direction: column; gap: 0.25rem; margin-bottom: 1rem; }
  .weak-key-row {
    display: flex; align-items: center; gap: 1rem; padding: 0.5rem 1rem;
    background: var(--bg-sub); border-radius: 4px; border: 1px solid var(--sub);
  }
  .weak-key-row.critical { border-color: var(--error); }
  .wk-char { font-weight: bold; color: var(--main); min-width: 1.5rem; }
  .wk-accuracy { color: var(--text); min-width: 4rem; }
  .wk-errors { color: var(--sub); font-size: 0.75rem; flex: 1; }
  .wk-rank { color: var(--sub); font-size: 0.75rem; }
  button {
    background-color: var(--bg-sub); color: var(--main); border: 1px solid var(--main);
    padding: 0.5rem 1.5rem; font-family: inherit; font-size: 0.875rem;
    cursor: pointer; border-radius: 4px; margin-bottom: 1rem;
  }
  button:hover { background-color: var(--main); color: var(--bg); }
</style>