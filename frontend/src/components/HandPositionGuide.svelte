<script lang="ts">
  // HandPositionGuide — схема рук с подсветкой нужного пальца.

  import { FINGERS, RU_FINGERS } from '../lib/keyboard';

  let {
    nextChar = '',
    isRussian = false,
  }: {
    nextChar?: string;
    isRussian?: boolean;
  } = $props();

  type Finger = {
    code: string;
    name: string;
    kind: 'pinky' | 'ring' | 'middle' | 'index' | 'thumb';
  };

  const leftFingers: Finger[] = [
    { code: 'LP', name: 'Left pinky', kind: 'pinky' },
    { code: 'LR', name: 'Left ring finger', kind: 'ring' },
    { code: 'LM', name: 'Left middle finger', kind: 'middle' },
    { code: 'LI', name: 'Left index finger', kind: 'index' },
    { code: 'LT', name: 'Left thumb', kind: 'thumb' },
  ];
  const rightFingers: Finger[] = [
    { code: 'RT', name: 'Right thumb', kind: 'thumb' },
    { code: 'RI', name: 'Right index finger', kind: 'index' },
    { code: 'RM', name: 'Right middle finger', kind: 'middle' },
    { code: 'RR', name: 'Right ring finger', kind: 'ring' },
    { code: 'RP', name: 'Right pinky', kind: 'pinky' },
  ];

  const fingers = $derived(isRussian ? RU_FINGERS : FINGERS);

  let activeFinger = $derived(nextChar === ' ' ? 'LT' : fingers[nextChar.toLowerCase()] || '');

  function isActive(finger: Finger): boolean {
    return activeFinger === finger.code;
  }
</script>

<div class="hand-guide" aria-live="polite">
  <div class="hands">
    <div class="hand left-hand" aria-label="Left hand">
      <div class="finger-row">
        {#each leftFingers as finger}
          <span class="finger finger--{finger.kind}" class:active={isActive(finger)} aria-label={finger.name}></span>
        {/each}
      </div>
      <div class="palm" aria-hidden="true"></div>
    </div>
    <div class="hand right-hand" aria-label="Right hand">
      <div class="finger-row">
        {#each rightFingers as finger}
          <span class="finger finger--{finger.kind}" class:active={isActive(finger)} aria-label={finger.name}></span>
        {/each}
      </div>
      <div class="palm" aria-hidden="true"></div>
    </div>
  </div>
  {#if nextChar}
    <div class="finger-info">Next: <strong>{nextChar === ' ' ? 'Space' : nextChar}</strong></div>
  {/if}
</div>

<style>
  .hand-guide { display: flex; flex-direction: column; align-items: center; gap: 0.5rem; }
  .hands { display: flex; gap: 1.5rem; align-items: flex-end; }
  .hand { width: 8.5rem; display: flex; flex-direction: column; align-items: center; }
  .finger-row { height: 3.75rem; display: flex; align-items: flex-end; gap: 0.18rem; }
  .finger {
    width: 1.2rem; border: 1px solid var(--sub); border-bottom: 0; border-radius: 0.55rem 0.55rem 0 0;
    background: var(--bg-sub); transition: background-color 150ms ease, border-color 150ms ease;
  }
  .finger--pinky { height: 2.35rem; }
  .finger--ring { height: 2.95rem; }
  .finger--middle { height: 3.45rem; }
  .finger--index { height: 3.15rem; }
  .finger--thumb { width: 1.45rem; height: 1.7rem; margin-bottom: 0.3rem; }
  .finger.active { background: var(--main); border-color: var(--main); }
  .palm { width: 100%; height: 2rem; border: 1px solid var(--sub); border-radius: 0.25rem 0.25rem 0.8rem 0.8rem; background: var(--bg-sub); }
  .finger-info { font-size: 0.875rem; color: var(--sub); }
  .finger-info strong { color: var(--main); margin-left: 0.25rem; }
  @media (max-width: 430px) { .hands { gap: 0.65rem; } .hand { width: 7.25rem; } .finger-row { gap: 0.12rem; } .finger { width: 1rem; } .finger--thumb { width: 1.25rem; } }
</style>
