<script lang="ts">
  // HandPositionGuide — схема рук с подсветкой нужного пальца.

  let {
    nextChar = '',
    isRussian = false,
  }: {
    nextChar?: string;
    isRussian?: boolean;
  } = $props();

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
    ф:'LP', я:'LP', ё:'LP',
    ы:'LR', ч:'LR', ц:'LR',
    в:'LM', с:'LM', у:'LM',
    а:'LI', п:'LI', к:'LI', м:'LI',
    о:'RI', л:'RI', д:'RI', р:'RI', т:'RI',
    е:'RM', г:'RM', ш:'RM',
    н:'RR', щ:'RR', з:'RR',
    ь:'RP', б:'RP', ю:'RP', ъ:'RP',
  };

  const labels: Record<string, string> = {
    LP: 'Pinky', LR: 'Ring', LM: 'Middle', LI: 'Index',
    RI: 'Index', RM: 'Middle', RR: 'Ring', RP: 'Pinky',
  };

  const fingers = $derived(isRussian ? RU_FINGERS : FINGERS);

  let activeFinger = $derived(nextChar ? fingers[nextChar.toLowerCase()] || '' : '');

  function isActive(finger: string): string {
    return activeFinger === finger ? 'active' : '';
  }

  function getActiveLabel(): string {
    if (!activeFinger) return '';
    const hand = activeFinger.startsWith('L') ? 'Left' : 'Right';
    return `${hand} ${labels[activeFinger]}`;
  }
</script>

<div class="hand-guide">
  <div class="hands">
    <div class="hand left-hand">
      <div class="finger {isActive('LP')}">Pinky</div>
      <div class="finger {isActive('LR')}">Ring</div>
      <div class="finger {isActive('LM')}">Middle</div>
      <div class="finger {isActive('LI')}">Index</div>
    </div>
    <div class="hand-sep"></div>
    <div class="hand right-hand">
      <div class="finger {isActive('RI')}">Index</div>
      <div class="finger {isActive('RM')}">Middle</div>
      <div class="finger {isActive('RR')}">Ring</div>
      <div class="finger {isActive('RP')}">Pinky</div>
    </div>
  </div>
  {#if nextChar && activeFinger}
    <div class="finger-info">
      <span class="target-key">{nextChar}</span>
      <span class="arrow">→</span>
      <span class="finger-name">{getActiveLabel()}</span>
    </div>
  {/if}
</div>

<style>
  .hand-guide { display: flex; flex-direction: column; align-items: center; gap: 0.5rem; }
  .hands { display: flex; gap: 2rem; align-items: center; }
  .hand { display: flex; gap: 0.25rem; }
  .hand-sep { width: 1px; height: 30px; background: var(--sub); opacity: 0.3; }
  .finger {
    padding: 0.25rem 0.75rem; border: 1px solid var(--sub); border-radius: 8px 8px 4px 4px;
    font-size: 0.65rem; color: var(--sub); background: var(--bg-sub); transition: all 0.15s;
  }
  .finger.active { background: var(--main); color: var(--bg); border-color: var(--main); transform: translateY(-4px); }
  .finger-info { font-size: 0.875rem; color: var(--text); display: flex; gap: 0.5rem; align-items: center; }
  .target-key { font-size: 1.5rem; color: var(--main); font-weight: bold; }
  .arrow { color: var(--sub); }
  .finger-name { color: var(--text); }
</style>