<script lang="ts">
  import { FINGERS, HOME_ROW_EN, HOME_ROW_RU, ROWS, RU_FINGERS, RU_ROWS } from '../lib/keyboard';

  type KeySpec = { label: string; value?: string; span: number; special?: boolean };
  type KeyReference = { label: string; value?: string };

  let { nextChar = '', isRussian = false, lastErrorChar = '', charStats = {} }: {
    nextChar?: string;
    isRussian?: boolean;
    lastErrorChar?: string;
    charStats?: Record<string, { correct: number; incorrect: number; total: number }>;
  } = $props();

  const SHIFTED_SYMBOL_KEYS: Record<string, string> = {
    '!': '1', '@': '2', '#': '3', '$': '4', '%': '5', '^': '6', '&': '7', '*': '8', '(': '9', ')': '0',
    '_': '-', '+': '=', '{': '[', '}': ']', '|': '\\', ':': ';', '"': "'", '<': ',', '>': '.', '?': '/',
  };
  // Group the function keys like a physical ANSI keyboard. The separate
  // navigation group is rendered in a column that exactly matches Ins/Home/PgUp.
  const functionKeyGroups = [
    ['Esc'],
    ['F1', 'F2', 'F3', 'F4'],
    ['F5', 'F6', 'F7', 'F8'],
    ['F9', 'F10', 'F11', 'F12'],
  ];
  const numberRow: KeySpec[] = [
    { label: '`', span: 1 }, ...['1','2','3','4','5','6','7','8','9','0','-','='].map(label => ({ label, span: 1 })),
    { label: 'Backspace', span: 2, special: true },
  ];
  const navTop = ['Ins', 'Home', 'PgUp'];
  const navMiddle = ['Del', 'End', 'PgDn'];

  const letters = $derived(isRussian ? RU_ROWS : ROWS);
  const homeRow = $derived(isRussian ? HOME_ROW_RU : HOME_ROW_EN);
  const fingers = $derived(isRussian ? RU_FINGERS : FINGERS);
  const topRow = $derived<KeySpec[]>([
    { label: 'Tab', span: 1.5, special: true },
    ...letters[0].map(label => ({ label, span: 1 })),
    ...(isRussian ? [{ label: '\\', span: 1 }] : [{ label: '[', span: 1 }, { label: ']', span: 1 }, { label: '\\', span: 1 }]),
  ]);
  const middleRow = $derived<KeySpec[]>([
    { label: 'Caps', span: 1.75, special: true },
    ...letters[1].map(label => ({ label, span: 1 })),
    // ROWS[1] already ends with `;` in the English layout; add only apostrophe.
    ...(isRussian ? [] : [{ label: "'", span: 1 }]),
    { label: 'Enter', span: 2.25, special: true },
  ]);
  const bottomRow = $derived<KeySpec[]>([
    { label: 'Shift', span: 2.25, special: true }, ...letters[2].map(label => ({ label, span: 1 })), { label: 'Shift', span: 2.75, special: true },
  ]);
  const spaceRow: KeySpec[] = [
    { label: 'Ctrl', span: 1.25, special: true }, { label: 'Win', span: 1.25, special: true }, { label: 'Alt', span: 1.25, special: true },
    { label: 'Space', value: ' ', span: 6.25, special: true }, { label: 'Alt', span: 1.25, special: true }, { label: 'Fn', span: 1.25, special: true },
    { label: 'Menu', span: 1.25, special: true }, { label: 'Ctrl', span: 1.25, special: true },
  ];

  function normalise(key: string): string {
    const lower = key === 'Space' ? ' ' : key.toLowerCase();
    return SHIFTED_SYMBOL_KEYS[lower] || lower;
  }
  function getKeyClass(key: KeyReference | string): string {
    const spec = typeof key === 'string' ? { label: key } : key;
    const value = normalise(spec.value ?? spec.label);
    const classes: string[] = [];
    if (homeRow.has(value)) classes.push('home-key');
    if (nextChar && value === normalise(nextChar)) classes.push('next-key');
    if (lastErrorChar && value === normalise(lastErrorChar)) classes.push('error-key');
    const stats = charStats[value];
    if (stats?.total) {
      const accuracy = (stats.correct / stats.total) * 100;
      if (accuracy < 70) classes.push('weak-critical');
      else if (accuracy < 90) classes.push('weak-warning');
      else classes.push('measured-key');
    }
    const finger = fingers[value] || '';
    if (finger.startsWith('L')) classes.push('left-hand');
    if (finger.startsWith('R')) classes.push('right-hand');
    return classes.join(' ');
  }
  function finger(key: KeyReference | string): string {
    const spec = typeof key === 'string' ? { label: key } : key;
    return fingers[normalise(spec.value ?? spec.label)] || '';
  }
</script>

<section class="keyboard-trainer" aria-label="Full keyboard trainer">
  <div class="keyboard-board" aria-live="polite">
    <div class="function-row">
      <div class="function-main" aria-label="Function keys">
        {#each functionKeyGroups as group}
          <div class="function-group">
            {#each group as key}<div class="key function-key">{key}</div>{/each}
          </div>
        {/each}
      </div>
      <div class="function-navigation" aria-label="System keys">
        <div class="key function-key">PrtSc</div><div class="key function-key">ScrLk</div><div class="key function-key">Pause</div>
      </div>
    </div>

    <div class="keyboard-body">
      <div class="main-cluster">
        <div class="main-row">
          {#each numberRow as key, index}<div class="key {getKeyClass(key)}" class:special={key.special} class:right-edge={index === numberRow.length - 1} style:flex={`0 0 calc(var(--u) * ${key.span})`} title={finger(key)}>{key.label}</div>{/each}
        </div>
        <div class="main-row">
          {#each topRow as key, index}<div class="key {getKeyClass(key)}" class:special={key.special} class:right-edge={index === topRow.length - 1} style:flex={`0 0 calc(var(--u) * ${key.span})`} title={finger(key)}>{key.label}</div>{/each}
        </div>
        <div class="main-row">
          {#each middleRow as key, index}<div class="key {getKeyClass(key)}" class:special={key.special} class:right-edge={index === middleRow.length - 1} style:flex={`0 0 calc(var(--u) * ${key.span})`} title={finger(key)}>{key.label}</div>{/each}
        </div>
        <div class="main-row">
          {#each bottomRow as key, index}<div class="key {getKeyClass(key)}" class:special={key.special} class:right-edge={index === bottomRow.length - 1} style:flex={`0 0 calc(var(--u) * ${key.span})`} title={finger(key)}>{key.label}</div>{/each}
        </div>
        <div class="main-row">
          {#each spaceRow as key, index}<div class="key {getKeyClass(key)}" class:special={key.special} class:right-edge={index === spaceRow.length - 1} style:flex={`0 0 calc(var(--u) * ${key.span})`} title={finger(key)}>{key.label}</div>{/each}
        </div>
      </div>

      <div class="navigation-cluster">
        <div class="nav-row">{#each navTop as key}<div class="key nav-key">{key}</div>{/each}</div>
        <div class="nav-row">{#each navMiddle as key}<div class="key nav-key">{key}</div>{/each}</div>
        <div class="arrows"><div></div><div class="key nav-key">↑</div><div></div><div class="key nav-key">←</div><div class="key nav-key">↓</div><div class="key nav-key">→</div></div>
      </div>

      <div class="numpad" aria-label="Numeric keypad">
        <div class="key num-key">Num</div><div class="key num-key">/</div><div class="key num-key">*</div><div class="key num-key">−</div>
        <div class="key num-key {getKeyClass('7')}">7</div><div class="key num-key {getKeyClass('8')}">8</div><div class="key num-key {getKeyClass('9')}">9</div><div class="key num-key num-plus">+</div>
        <div class="key num-key {getKeyClass('4')}">4</div><div class="key num-key {getKeyClass('5')}">5</div><div class="key num-key {getKeyClass('6')}">6</div>
        <div class="key num-key {getKeyClass('1')}">1</div><div class="key num-key {getKeyClass('2')}">2</div><div class="key num-key {getKeyClass('3')}">3</div><div class="key num-key num-enter">Enter</div>
        <div class="key num-key num-zero {getKeyClass('0')}">0</div><div class="key num-key {getKeyClass('.')}">.</div>
      </div>
    </div>
  </div>
  {#if nextChar}<div class="next-key-info">Next: <strong>{nextChar === ' ' ? 'Space' : nextChar}</strong> <span>{finger(nextChar)}</span></div>{/if}
</section>

<style>
  .keyboard-trainer { width: 100%; overflow-x: auto; padding: 0.5rem 0.25rem; display: grid; place-items: center; }
  .keyboard-board { --u: 40px; --gap: 4px; min-width: 1055px; display: flex; flex-direction: column; gap: 14px; }
  .function-row { display: flex; align-items: flex-start; gap: 18px; }
  .function-main { width: calc(var(--u) * 15 + var(--gap) * 14); display: flex; justify-content: space-between; }
  .function-group { display: flex; gap: var(--gap); }
  .function-navigation { width: calc(var(--u) * 3 + var(--gap) * 2); display: grid; grid-template-columns: repeat(3, var(--u)); gap: var(--gap); }
  .keyboard-body { display: flex; align-items: flex-start; gap: 18px; }
  .main-cluster { width: calc(var(--u) * 15 + var(--gap) * 14); display: flex; flex-direction: column; gap: var(--gap); }
  .main-row { display: flex; gap: var(--gap); min-height: 44px; }
  .navigation-cluster { width: calc(var(--u) * 3 + var(--gap) * 2); display: flex; flex-direction: column; gap: var(--gap); }
  .nav-row, .arrows { display: grid; grid-template-columns: repeat(3, var(--u)); gap: var(--gap); min-height: 44px; }
  .arrows { margin-top: 44px; }
  .numpad { display: grid; grid-template-columns: repeat(4, var(--u)); grid-template-rows: repeat(5, 44px); gap: var(--gap); }
  .key { min-width: 0; height: 44px; display: grid; place-items: center; background: var(--color-key-background); color: var(--text); border: 1px solid var(--color-key-border); border-radius: 5px; font-size: 0.72rem; font-weight: 700; white-space: nowrap; transition: transform 120ms ease, background-color 120ms ease, border-color 120ms ease; }
  .function-key { width: var(--u); height: 32px; font-size: 0.62rem; }
  .special { color: var(--sub); font-size: 0.62rem; }
  /* Let the final special key fill only its row's remaining physical space.
     This keeps its right edge aligned without introducing a detached gap on
     the left (notably before Slash, Enter, Shift, and the right Ctrl). */
  .main-row > .right-edge { flex-grow: 1 !important; }
  .next-key { background: var(--main); border-color: var(--main); color: var(--color-accent-text); box-shadow: 0 0 0 3px color-mix(in srgb, var(--main) 25%, transparent); transform: translateY(-3px) scale(1.04); z-index: 1; }
  .error-key { background: color-mix(in srgb, var(--error) 22%, var(--bg-sub)); border-color: var(--error); }
  .weak-critical { background: color-mix(in srgb, var(--error) 26%, var(--bg-sub)); border-color: var(--error); color: var(--text); }
  .weak-warning { background: color-mix(in srgb, var(--color-warning) 22%, var(--bg-sub)); border-color: var(--color-warning); color: var(--text); }
  .measured-key { border-color: color-mix(in srgb, var(--main) 45%, var(--sub)); }
  .key.next-key { background: var(--main); border-color: var(--main); color: var(--color-accent-text); }
  .num-key { width: var(--u); }
  .num-plus { grid-column: 4; grid-row: 2 / span 2; height: auto; }
  .num-enter { grid-column: 4; grid-row: 4 / span 2; height: auto; font-size: 0.62rem; }
  .num-zero { grid-column: 1 / span 2; width: auto; }
  .next-key-info { margin-top: 0.5rem; color: var(--sub); font-size: 0.78rem; }
  .next-key-info strong { color: var(--main); margin: 0 0.35rem; }
  @media (max-width: 1150px) { .keyboard-board { --u: 34px; min-width: 900px; } .key, .main-row, .nav-row, .arrows { height: 38px; min-height: 38px; } .numpad { grid-template-rows: repeat(5, 38px); } .arrows { margin-top: 38px; } }
</style>
