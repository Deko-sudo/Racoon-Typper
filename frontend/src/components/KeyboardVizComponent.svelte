<script lang="ts">
  // KeyboardViz — визуальная клавиатура с heatmap intensity.
  // QWERTY layout, color-coded by error count.

  let { heatmap = {}, charStats = {} }: {
    heatmap?: Record<string, { total_attempts: number; correct: number; incorrect: number }>;
    charStats?: Record<string, { correct: number; incorrect: number; total: number }>;
  } = $props();

  const ROWS = [
    ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
    ['z', 'x', 'c', 'v', 'b', 'n', 'm'],
  ];

  // Finger assignments (simplified)
  const FINGERS: Record<string, string> = {
    q: 'LP', a: 'LP', z: 'LP',
    w: 'LR', s: 'LR', x: 'LR',
    e: 'LM', d: 'LM', c: 'LM',
    r: 'LI', f: 'LI', v: 'LI', t: 'LI', g: 'LI', b: 'LI',
    y: 'RI', h: 'RI', n: 'RI', u: 'RI', j: 'RI', m: 'RI',
    i: 'RM', k: 'RM', ',': 'RM',
    o: 'RR', l: 'RR', '.': 'RR',
    p: 'RP', ';': 'RP', '/': 'RP',
  };

  function getKeyData(key: string): { correct: number; incorrect: number; total: number } {
    // Try charStats first, then heatmap
    if (charStats[key]) return charStats[key];
    if (heatmap[key]) {
      return {
        correct: heatmap[key].correct,
        incorrect: heatmap[key].incorrect,
        total: heatmap[key].total_attempts,
      };
    }
    return { correct: 0, incorrect: 0, total: 0 };
  }

  function getKeyColor(key: string): string {
    const data = getKeyData(key);
    if (data.total === 0) return 'var(--sub)';
    const accuracy = (data.correct / data.total) * 100;
    if (accuracy >= 95) return 'var(--text)';
    if (accuracy >= 80) return '#e2b714';
    if (accuracy >= 60) return '#ff8c42';
    return 'var(--error)';
  }

  function getKeyIntensity(key: string): number {
    const data = getKeyData(key);
    if (data.total === 0) return 0.3;
    return Math.min(1, data.total / 20);
  }

  function getKeyLabel(key: string): string {
    const data = getKeyData(key);
    if (data.total === 0) return '';
    const acc = ((data.correct / data.total) * 100).toFixed(0);
    return `${acc}% (${data.incorrect}e)`;
  }

  function getFinger(key: string): string {
    return FINGERS[key] || '';
  }
</script>

<div class="keyboard-viz">
  <h3>Keyboard Heatmap</h3>
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
            <span class="key-finger">{getFinger(key)}</span>
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .keyboard-viz { max-width: 700px; width: 100%; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; text-align: center; }
  .keyboard { display: flex; flex-direction: column; gap: 0.25rem; align-items: center; }
  .keyboard-row { display: flex; gap: 0.25rem; }
  .key {
    width: 44px; height: 48px; border: 1px solid var(--bg-sub); border-radius: 4px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: var(--bg-sub); font-size: 0.75rem; transition: all 0.2s; position: relative;
  }
  .key-char { font-weight: bold; font-size: 0.9rem; }
  .key-acc { font-size: 0.55rem; opacity: 0.8; }
  .key-finger { font-size: 0.5rem; opacity: 0.5; position: absolute; bottom: 2px; right: 3px; }
</style>