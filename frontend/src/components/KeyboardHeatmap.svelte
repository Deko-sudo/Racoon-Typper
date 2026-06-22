<script lang="ts">
  // QWERTY keyboard layout rows
  const ROWS = [
    ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
    ['z', 'x', 'c', 'v', 'b', 'n', 'm'],
  ];

  let { heatmap = {} }: { heatmap?: Record<string, { total_attempts: number; correct: number; incorrect: number; avg_wpm_at_key: number }> } = $props();

  function getKeyColor(key: string): string {
    const data = heatmap[key];
    if (!data || data.total_attempts === 0) return 'var(--sub)';
    const accuracy = (data.correct / data.total_attempts) * 100;
    if (accuracy >= 95) return 'var(--text)';
    if (accuracy >= 80) return '#e2b714';
    if (accuracy >= 60) return '#ff8c42';
    return 'var(--error)';
  }

  function getKeyIntensity(key: string): number {
    const data = heatmap[key];
    if (!data || data.total_attempts === 0) return 0.3;
    return Math.min(1, data.total_attempts / 20);
  }

  function getKeyLabel(key: string): string {
    const data = heatmap[key];
    if (!data || data.total_attempts === 0) return '';
    const acc = ((data.correct / data.total_attempts) * 100).toFixed(0);
    return `${acc}%`;
  }
</script>

<div class="heatmap-container">
  <h3>Heatmap</h3>
  <div class="keyboard">
    {#each ROWS as row, rowIdx}
      <div class="keyboard-row" style="margin-left: {rowIdx * 20}px;">
        {#each row as key}
          <div
            class="key"
            style="color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};"
            title="{key}: {getKeyLabel(key)}"
          >
            <span class="key-char">{key}</span>
            {#if getKeyLabel(key)}
              <span class="key-acc">{getKeyLabel(key)}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .heatmap-container { max-width: 700px; width: 100%; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; text-align: center; }
  .keyboard { display: flex; flex-direction: column; gap: 0.25rem; align-items: center; }
  .keyboard-row { display: flex; gap: 0.25rem; }
  .key {
    width: 40px; height: 40px; border: 1px solid var(--bg-sub); border-radius: 4px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: var(--bg-sub); font-size: 0.75rem; transition: all 0.2s;
  }
  .key-char { font-weight: bold; }
  .key-acc { font-size: 0.6rem; opacity: 0.8; }
</style>