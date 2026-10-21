<script lang="ts">
  // HandPositionGuide — схема рук с подсветкой нужного пальца.

  import { fingerForKey } from '../lib/keyboard';

  let {
    nextChar = '',
    isRussian = false,
    keyboardLayout = 'qwerty',
  }: {
    nextChar?: string;
    isRussian?: boolean;
    keyboardLayout?: string;
  } = $props();

  // Physical position decides the finger: the character is resolved against
  // the active layout's key positions (Cyrillic always maps to JCUKEN), so
  // A/Ф light up the left pinky, S/Ы the left ring, and so on. Unknown keys
  // yield '' and leave both hands un-highlighted.
  let activeFinger = $derived(fingerForKey(nextChar, keyboardLayout, isRussian));
</script>

<div class="hand-guide" aria-live="polite">
  <div class="hands">
    <svg class="hand" viewBox="0 0 200 180" role="img" aria-label="Left hand typing guide">
      <title>Left hand</title>
      <defs>
        <path id="left-hand-shape" d="M55 176c-14-4-23-14-26-29l-7-36c-2-11-1-22 3-32 3-8 10-13 18-12 8 1 13 8 13 17l1 8-3-44c-1-11 5-20 15-21 10 0 17 8 17 19l1 40-1-63C86 11 93 3 103 3s17 8 17 20l-2 64 2-47c0-11 7-19 17-19s16 9 15 20l-4 58 11-25c4-10 14-14 22-10 9 4 12 14 8 23l-20 46c-5 11-9 20-10 30l-1 13Z" />
        <clipPath id="left-hand-clip"><use href="#left-hand-shape" /></clipPath>
      </defs>
      <use href="#left-hand-shape" class="hand-fill" />
      <g clip-path="url(#left-hand-clip)">
        <!-- Highlight bands: x-ranges are bounded by the inter-finger valleys
             measured from the rendered hand path; bottoms extend to the
             finger knuckle lines (staggered: pinky 80, ring 94, middle 98,
             index 92, thumb 88) — the full finger fills, the palm stays
             untouched. -->
        <path class="highlight" class:active={activeFinger === 'LP'} d="M20 40h36v40H20Z" />
        <path class="highlight" class:active={activeFinger === 'LR'} d="M56 14h34v80H56Z" />
        <path class="highlight" class:active={activeFinger === 'LM'} d="M90 -4h32v102H90Z" />
        <path class="highlight" class:active={activeFinger === 'LI'} d="M122 10h30v82H122Z" />
        <path class="highlight" class:active={activeFinger === 'LT'} d="M150 44h48v44H150Z" />
      </g>
      <use href="#left-hand-shape" class="hand-outline" />
      <path class="palm-line" d="M54 145c22 7 58 7 82-1M139 116c-13 5-22 13-27 25" />
    </svg>
    <svg class="hand" viewBox="0 0 200 180" role="img" aria-label="Right hand typing guide">
      <title>Right hand</title>
      <defs>
        <path id="right-hand-shape" d="M55 176c-14-4-23-14-26-29l-7-36c-2-11-1-22 3-32 3-8 10-13 18-12 8 1 13 8 13 17l1 8-3-44c-1-11 5-20 15-21 10 0 17 8 17 19l1 40-1-63C86 11 93 3 103 3s17 8 17 20l-2 64 2-47c0-11 7-19 17-19s16 9 15 20l-4 58 11-25c4-10 14-14 22-10 9 4 12 14 8 23l-20 46c-5 11-9 20-10 30l-1 13Z" />
        <clipPath id="right-hand-clip"><use href="#right-hand-shape" /></clipPath>
      </defs>
      <g transform="translate(200 0) scale(-1 1)">
        <use href="#right-hand-shape" class="hand-fill" />
        <g clip-path="url(#right-hand-clip)">
          <path class="highlight" class:active={activeFinger === 'RP'} d="M20 40h36v40H20Z" />
          <path class="highlight" class:active={activeFinger === 'RR'} d="M56 14h34v80H56Z" />
          <path class="highlight" class:active={activeFinger === 'RM'} d="M90 -4h32v102H90Z" />
          <path class="highlight" class:active={activeFinger === 'RI'} d="M122 10h30v82H122Z" />
          <path class="highlight" class:active={activeFinger === 'RT'} d="M150 44h48v44H150Z" />
        </g>
        <use href="#right-hand-shape" class="hand-outline" />
        <path class="palm-line" d="M54 145c22 7 58 7 82-1M139 116c-13 5-22 13-27 25" />
      </g>
    </svg>
  </div>
  {#if nextChar}
    <div class="finger-info">Next: <strong>{nextChar === ' ' ? 'Space' : nextChar}</strong></div>
  {/if}
</div>

<style>
  .hand-guide { display: flex; flex-direction: column; align-items: center; gap: 0.5rem; }
  .hands { display: flex; gap: 1rem; align-items: center; }
  .hand { width: 9rem; height: auto; overflow: visible; }
  .hand-fill { fill: var(--bg-sub); }
  .hand-outline { fill: none; stroke: var(--sub); stroke-width: 2.2; stroke-linecap: round; stroke-linejoin: round; }
  .highlight { fill: transparent; transition: fill 150ms ease; }
  .highlight.active { fill: var(--main); }
  .palm-line { fill: none; stroke: color-mix(in srgb, var(--sub) 60%, transparent); stroke-width: 2; stroke-linecap: round; }
  .finger-info { font-size: 0.875rem; color: var(--sub); }
  .finger-info strong { color: var(--main); margin-left: 0.25rem; }
  @media (max-width: 430px) { .hands { gap: 0.25rem; } .hand { width: 7.6rem; } }
</style>
