<script lang="ts">
  import { FINGERS, ROWS, RU_ROWS } from '../lib/keyboard';

  let {
    heatmap = {},
    charStats = {},
  }: {
    heatmap?: Record<string, { total_attempts: number; correct: number; incorrect: number; avg_wpm_at_key: number }>;
    charStats?: Record<string, { correct: number; incorrect: number; total: number }>;
  } = $props();

  // Автоопределение раскладки по данным: если в stats заметно кириллических
  // клавиш больше, чем латинских — рендерим ЙЦУКЕН-ряды. Раньше RU-сессии
  // рендерили пустую QWERTY-клавиатуру (heatmap-ключи не совпадали).
  const statsSource = $derived(Object.keys(charStats).length > 0 ? charStats : Object.fromEntries(
    Object.entries(heatmap).map(([k, v]) => [k, { correct: v.correct, incorrect: v.incorrect, total: v.total_attempts }]),
  ));

  let isCyrillic = $derived.by(() => {
    let cyr = 0, lat = 0;
    for (const key of Object.keys(statsSource)) {
      if (/[а-яА-ЯёЁ]/.test(key)) cyr++;
      else if (/[a-zA-Z]/.test(key)) lat++;
    }
    return cyr > lat;
  });

  const heatmapRows = $derived(
    (isCyrillic ? RU_ROWS : ROWS).map((row) => row.filter((key) => key.length === 1)),
  );

  // Case-insensitive агрегация: бэкенд ключует точным символом ('A' и 'a' —
  // разные записи), а клавиша на раскладке одна. Складываем обе вариации.
  function getKeyData(key: string): { correct: number; incorrect: number; total: number } {
    const lower = key.toLowerCase();
    const upper = key.toUpperCase();
    const direct = charStats[key] || heatmap[key];
    const lowerEntry = charStats[lower] || heatmap[lower];
    const upperEntry = charStats[upper] || heatmap[upper];
    const entries = [direct, lowerEntry, upperEntry].filter(Boolean) as Array<{ correct: number; incorrect: number; total?: number; total_attempts?: number }>;
    if (entries.length === 0) return { correct: 0, incorrect: 0, total: 0 };
    return {
      correct: entries.reduce((s, e) => s + e.correct, 0),
      incorrect: entries.reduce((s, e) => s + e.incorrect, 0),
      total: entries.reduce((s, e) => s + (e.total ?? e.total_attempts ?? 0), 0),
    };
  }

  function getFinger(key: string): string {
    return FINGERS[key] || '';
  }

  // Staggered offsets per row: ANSI stagger 0.25u/0.5u при 40px-клавише и
  // 4px-зазоре (44px pitch) = 11/22px.
  const ROW_STAGGER = [0, 11, 22]; // top, home, bottom (px)

  function getKeyColor(key: string): string {
    const data = getKeyData(key);
    if (data.total === 0) return 'var(--sub)';
    const accuracy = (data.correct / data.total) * 100;
    if (accuracy >= 95) return 'var(--text)';
    if (accuracy >= 80) return 'var(--color-chart-positive)';
    if (accuracy >= 60) return 'var(--color-warning)';
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
    return `${acc}%`;
  }
</script>

<div class="heatmap-container">
  <h3>Heatmap</h3>
  <div class="keyboard">
    {#each heatmapRows as row, rowIdx}
      <div class="keyboard-row" style="padding-left: {ROW_STAGGER[rowIdx] || 0}px;">
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
  .heatmap-container { max-width: 700px; width: 100%; display: flex; flex-direction: column; align-items: center; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; text-align: center; }
  .keyboard {
    display: flex; flex-direction: column; gap: 0.25rem;
    align-items: center; width: 100%;
  }
  .keyboard-row { display: flex; gap: 0.25rem; justify-content: center; }
  .key {
    width: 40px; height: 40px; border: 1px solid var(--color-key-border); border-radius: 4px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: var(--color-key-background); font-size: 0.75rem; transition: all 0.2s; position: relative;
    flex-shrink: 0;
  }
  .key-char { font-weight: bold; }
  .key-acc { font-size: 0.6rem; opacity: 0.8; }
  .key-finger { font-size: 0.5rem; opacity: 0.5; position: absolute; bottom: 2px; right: 3px; }
</style>
