<script lang="ts">
  // KeyboardViz — полная клавиатура с heatmap intensity.
  // QWERTY layout + все клавиши + numpad.

  let { heatmap = {}, charStats = {} }: {
    heatmap?: Record<string, { total_attempts: number; correct: number; incorrect: number }>;
    charStats?: Record<string, { correct: number; incorrect: number; total: number }>;
  } = $props();

  // Row 1: function row
  const FUNC_ROW = ['esc', 'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12'];

  // Row 2: number row
  const NUM_ROW = ['`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', 'del'];

  // Row 3: top row
  const TOP_ROW = ['tab', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\\'];

  // Row 4: home row
  const HOME_ROW = ['caps', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', "'", 'enter'];

  // Row 5: bottom row
  const BOTTOM_ROW = ['shift', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 'shift'];

  // Row 6: space row
  const SPACE_ROW = ['ctrl', 'win', 'alt', 'space', 'alt', 'fn', 'menu', 'ctrl'];

  // Numpad
  const NUMPAD_ROWS = [
    ['num', '/', '*', '-'],
    ['7', '8', '9', '+'],
    ['4', '5', '6'],
    ['1', '2', '3', 'enter'],
    ['0', '.'],
  ];

  // Special key widths
  const KEY_WIDTHS: Record<string, string> = {
    'esc': '40px', 'tab': '64px', 'caps': '72px', 'enter': '78px',
    'shift': '96px', 'ctrl': '56px', 'win': '48px', 'alt': '48px',
    'space': '200px', 'fn': '40px', 'menu': '40px', 'del': '56px',
    'num': '48px',
  };

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

  function getKeyWidth(key: string): string {
    return KEY_WIDTHS[key] || '48px';
  }

  function isSpecial(key: string): boolean {
    return !!KEY_WIDTHS[key];
  }
</script>

<div class="keyboard-viz">
  <h3>Keyboard Heatmap</h3>
  <div class="keyboard-wrapper">
    <div class="keyboard-main">
      <!-- Function row -->
      <div class="keyboard-row">
        {#each FUNC_ROW as key}
          <div class="key key-fn" style="color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};" title="{key}">
            <span class="key-char-sm">{key}</span>
          </div>
        {/each}
      </div>
      <!-- Number row -->
      <div class="keyboard-row">
        {#each NUM_ROW as key}
          <div
            class="key {isSpecial(key) ? 'key-special' : ''}"
            style="width: {getKeyWidth(key)}; color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};"
            title="{key}: {getKeyLabel(key)}"
          >
            <span class="key-char">{key}</span>
            {#if getKeyLabel(key) && !isSpecial(key)}
              <span class="key-acc">{getKeyLabel(key)}</span>
            {/if}
            {#if !isSpecial(key)}
              <span class="key-finger">{getFinger(key)}</span>
            {/if}
          </div>
        {/each}
      </div>
      <!-- Top row -->
      <div class="keyboard-row">
        {#each TOP_ROW as key}
          <div
            class="key {isSpecial(key) ? 'key-special' : ''}"
            style="width: {getKeyWidth(key)}; color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};"
            title="{key}: {getKeyLabel(key)}"
          >
            <span class="key-char">{key}</span>
            {#if getKeyLabel(key) && !isSpecial(key)}
              <span class="key-acc">{getKeyLabel(key)}</span>
            {/if}
            {#if !isSpecial(key)}
              <span class="key-finger">{getFinger(key)}</span>
            {/if}
          </div>
        {/each}
      </div>
      <!-- Home row -->
      <div class="keyboard-row">
        {#each HOME_ROW as key}
          <div
            class="key {isSpecial(key) ? 'key-special' : ''}"
            style="width: {getKeyWidth(key)}; color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};"
            title="{key}: {getKeyLabel(key)}"
          >
            <span class="key-char">{key}</span>
            {#if getKeyLabel(key) && !isSpecial(key)}
              <span class="key-acc">{getKeyLabel(key)}</span>
            {/if}
            {#if !isSpecial(key)}
              <span class="key-finger">{getFinger(key)}</span>
            {/if}
          </div>
        {/each}
      </div>
      <!-- Bottom row -->
      <div class="keyboard-row">
        {#each BOTTOM_ROW as key}
          <div
            class="key {isSpecial(key) ? 'key-special' : ''}"
            style="width: {getKeyWidth(key)}; color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};"
            title="{key}: {getKeyLabel(key)}"
          >
            <span class="key-char">{key}</span>
            {#if getKeyLabel(key) && !isSpecial(key)}
              <span class="key-acc">{getKeyLabel(key)}</span>
            {/if}
            {#if !isSpecial(key)}
              <span class="key-finger">{getFinger(key)}</span>
            {/if}
          </div>
        {/each}
      </div>
      <!-- Space row -->
      <div class="keyboard-row">
        {#each SPACE_ROW as key}
          <div
            class="key key-special"
            style="width: {getKeyWidth(key)}; color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};"
            title="{key}"
          >
            <span class="key-char-sm">{key}</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Numpad -->
    <div class="keyboard-numpad">
      {#each NUMPAD_ROWS as row}
        <div class="numpad-row">
          {#each row as key}
            <div
              class="key {isSpecial(key) ? 'key-special' : ''}"
              style="width: {getKeyWidth(key)}; color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};"
              title="{key}: {getKeyLabel(key)}"
            >
              <span class="key-char">{key}</span>
            </div>
          {/each}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .keyboard-viz { width: 100%; display: flex; flex-direction: column; align-items: center; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; text-align: center; }
  .keyboard-wrapper { display: flex; gap: 1rem; justify-content: center; align-items: flex-start; }
  .keyboard-main { display: flex; flex-direction: column; gap: 0.25rem; align-items: center; }
  .keyboard-row { display: flex; gap: 0.25rem; }
  .keyboard-numpad { display: flex; flex-direction: column; gap: 0.25rem; align-items: center; margin-top: 28px; }
  .numpad-row { display: flex; gap: 0.25rem; }
  .key {
    height: 52px; border: 1px solid var(--bg-sub); border-radius: 4px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: var(--bg-sub); font-size: 0.75rem; transition: all 0.2s; position: relative;
  }
  .key-fn { height: 32px; width: 40px; }
  .key-special { justify-content: center; }
  .key-char { font-weight: bold; font-size: 0.9rem; }
  .key-char-sm { font-weight: bold; font-size: 0.7rem; }
  .key-acc { font-size: 0.55rem; opacity: 0.8; }
  .key-finger { font-size: 0.5rem; opacity: 0.5; position: absolute; bottom: 2px; right: 3px; }
</style>