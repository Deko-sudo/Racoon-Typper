<script lang="ts">
  // KeyboardViz — полная клавиатура с heatmap intensity.
  // QWERTY layout + все клавиши + numpad.

  import { FINGERS } from '../lib/keyboard';

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

  // Special key widths
  const KEY_WIDTHS: Record<string, string> = {
    'esc': '40px', 'tab': '64px', 'caps': '72px', 'enter': '78px',
    'shift': '96px', 'ctrl': '56px', 'win': '48px', 'alt': '48px',
    'space': '200px', 'fn': '40px', 'menu': '40px', 'del': '56px',
    'num': '48px',
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
      <div class="keyboard-row row-fn">
        {#each FUNC_ROW as key}
          <div class="key key-fn" style="color: {getKeyColor(key)}; opacity: {getKeyIntensity(key)};" title="{key}">
            <span class="key-char-sm">{key}</span>
          </div>
        {/each}
      </div>
      <!-- Number row -->
      <div class="keyboard-row row-number">
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
      <div class="keyboard-row row-top">
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
      <div class="keyboard-row row-home">
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
      <div class="keyboard-row row-bottom">
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
      <div class="keyboard-row row-space">
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

    <!-- Numpad: CSS grid preserves the physical geometry of +, Enter, and 0. -->
    <div class="keyboard-numpad">
      <div class="key key-special num-lock">num</div><div class="key">/</div><div class="key">*</div><div class="key">−</div>
      <div class="key">7</div><div class="key">8</div><div class="key">9</div><div class="key num-plus">+</div>
      <div class="key">4</div><div class="key">5</div><div class="key">6</div>
      <div class="key">1</div><div class="key">2</div><div class="key">3</div><div class="key key-special num-enter">enter</div>
      <div class="key num-zero">0</div><div class="key">.</div>
    </div>
  </div>
</div>

<style>
  .keyboard-viz { width: 100%; display: flex; flex-direction: column; align-items: center; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; text-align: center; }
  .keyboard-wrapper { display: flex; gap: 1rem; justify-content: center; align-items: flex-start; max-width: 100%; overflow-x: auto; padding: 0.25rem; }
  .keyboard-main { display: flex; flex-direction: column; gap: 0.25rem; align-items: flex-start; min-width: 748px; }
  .keyboard-row { display: flex; gap: 0.25rem; }
  .row-fn { align-self: flex-start; }
  /* The letter columns of each row are aligned to a single vertical line.
     The padding compensates for the differing widths of the leading special
     keys (number `` = 48px, tab = 64px, caps = 72px, left shift = 96px), each
     followed by a 4px gap. Without this, the home row (a/f/j/l) drifts right
     relative to the rows above and below. */
  .row-number { padding-left: 60px; }
  .row-top { padding-left: 44px; }
  .row-home { padding-left: 36px; }
  .row-bottom { padding-left: 12px; }
  .row-space { padding-left: 72px; }
  .keyboard-numpad { display: grid; grid-template-columns: repeat(4, 48px); grid-template-rows: repeat(5, 52px); gap: 0.25rem; margin-top: 36px; }
  .key {
    height: 52px; border: 1px solid var(--color-key-border); border-radius: 4px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: var(--color-key-background); font-size: 0.75rem; transition: all 0.2s; position: relative;
  }
  .key-fn { height: 32px; width: 40px; }
  .key-special { justify-content: center; }
  .key-char { font-weight: bold; font-size: 0.9rem; }
  .key-char-sm { font-weight: bold; font-size: 0.7rem; }
  .key-acc { font-size: 0.55rem; opacity: 0.8; }
  .key-finger { font-size: 0.5rem; opacity: 0.5; position: absolute; bottom: 2px; right: 3px; }
  .num-lock { width: 48px; }
  .num-plus { grid-column: 4; grid-row: 2 / span 2; width: 48px !important; height: auto; }
  .num-enter { grid-column: 4; grid-row: 4 / span 2; width: 48px !important; height: auto; }
  .num-zero { grid-column: 1 / span 2; width: auto !important; }
</style>
