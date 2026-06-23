<script lang="ts">
  // KeyboardTrainer — постоянная клавиатура под текстом.
  // Подсвечивает следующую клавишу, палец, Home Row.

  let {
    nextChar = '',
    isRussian = false,
    lastErrorChar = '',
  }: {
    nextChar?: string;
    isRussian?: boolean;
    lastErrorChar?: string;
  } = $props();

  // QWERTY layout
  const ROWS = [
    ['q','w','e','r','t','y','u','i','o','p'],
    ['a','s','d','f','g','h','j','k','l',';'],
    ['z','x','c','v','b','n','m',',','.','/'],
  ];

  // JCUKEN layout
  const RU_ROWS = [
    ['й','ц','у','к','е','н','г','ш','щ','з','х','ъ'],
    ['ф','ы','в','а','п','р','о','л','д','ж','э'],
    ['я','ч','с','м','и','т','ь','б','ю','.'],
  ];

  const HOME_ROW_EN = new Set(['a','s','d','f','g','h','j','k','l',';']);
  const HOME_ROW_RU = new Set(['ф','ы','в','а','п','р','о','л','д','ж','э']);

  // Finger mapping
  const FINGERS: Record<string, string> = {
    q:'LP', a:'LP', z:'LP',
    w:'LR', s:'LR', x:'LR',
    e:'LM', d:'LM', c:'LM',
    r:'LI', f:'LI', v:'LI', t:'LI', g:'LI', b:'LI',
    y:'RI', h:'RI', n:'RI', u:'RI', j:'RI', m:'RI',
    i:'RM', k:'RM', ',':'RM',
    o:'RR', l:'RR', '.':'RR',
    p:'RP', ';':'RP', '/':'RP',
  };

  const RU_FINGERS: Record<string, string> = {
    ф:'LP', я:'LP',
    ы:'LR', ч:'LR', ц:'LR',
    в:'LM', с:'LM', у:'LM',
    а:'LI', п:'LI', к:'LI', м:'LI',
    о:'RI', л:'RI', д:'RI', р:'RI', т:'RI',
    е:'RM', г:'RM', ш:'RM',
    н:'RR', щ:'RR', з:'RR',
    ь:'RP', б:'RP', ю:'RP', ъ:'RP',
  };

  const rows = $derived(isRussian ? RU_ROWS : ROWS);
  const homeRow = $derived(isRussian ? HOME_ROW_RU : HOME_ROW_EN);
  const fingers = $derived(isRussian ? RU_FINGERS : FINGERS);

  function getKeyClass(key: string): string {
    const classes: string[] = [];
    const lower = key.toLowerCase();

    if (homeRow.has(lower)) classes.push('home-row');
    if (nextChar && lower === nextChar.toLowerCase()) classes.push('next-key');
    if (lastErrorChar && lower === lastErrorChar.toLowerCase()) classes.push('error-key');

    const finger = fingers[lower] || '';
    if (finger.startsWith('L')) classes.push('left-hand');
    if (finger.startsWith('R')) classes.push('right-hand');

    return classes.join(' ');
  }

  function getFingerLabel(key: string): string {
    const f = fingers[key.toLowerCase()] || '';
    const labels: Record<string, string> = {
      LP: 'L-Pinky', LR: 'L-Ring', LM: 'L-Middle', LI: 'L-Index',
      RI: 'R-Index', RM: 'R-Middle', RR: 'R-Ring', RP: 'R-Pinky',
    };
    return labels[f] || '';
  }
</script>

<div class="keyboard-trainer">
  <div class="keyboard">
    {#each rows as row, rowIdx}
      <div class="keyboard-row" style="margin-left: {rowIdx * 20}px;">
        {#each row as key}
          <div class="key {getKeyClass(key)}" title={getFingerLabel(key)}>
            <span class="key-char">{key}</span>
            <span class="key-finger">{getFingerLabel(key)}</span>
          </div>
        {/each}
      </div>
    {/each}
  </div>
  {#if nextChar}
    <div class="next-key-info">
      Next: <span class="next-char">{nextChar}</span>
      <span class="next-finger">{getFingerLabel(nextChar)}</span>
    </div>
  {/if}
</div>

<style>
  .keyboard-trainer { display: flex; flex-direction: column; align-items: center; gap: 0.5rem; }
  .keyboard { display: flex; flex-direction: column; gap: 0.25rem; align-items: center; }
  .keyboard-row { display: flex; gap: 0.25rem; }
  .key {
    width: 40px; height: 44px; border: 1px solid var(--bg-sub); border-radius: 4px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: var(--bg-sub); transition: all 0.15s; position: relative;
  }
  .key.home-row { border-bottom: 2px solid var(--sub); }
  .key.next-key { background: var(--main); color: var(--bg); border-color: var(--main); transform: scale(1.15); }
  .key.error-key { border-color: var(--error); background: rgba(202,71,84,0.2); }
  .key.left-hand { border-left: 2px solid rgba(122,162,247,0.3); }
  .key.right-hand { border-right: 2px solid rgba(226,183,20,0.3); }
  .key-char { font-size: 0.85rem; font-weight: bold; }
  .key-finger { font-size: 0.5rem; opacity: 0.5; }
  .next-key-info { font-size: 0.75rem; color: var(--sub); display: flex; gap: 0.5rem; align-items: center; }
  .next-char { color: var(--main); font-weight: bold; font-size: 1rem; }
  .next-finger { color: var(--text); }
</style>