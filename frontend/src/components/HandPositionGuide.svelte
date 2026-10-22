<script lang="ts">
  import { fingerForKey } from '../lib/keyboard';
  import { HAND_FINGERS, HAND_OUTLINE } from '../lib/handGuide';
  import { t } from '../lib/i18n';

  let {
    nextChar = '', isRussian = false, keyboardLayout = 'qwerty', uiLang = 'en',
  }: {
    nextChar?: string;
    isRussian?: boolean;
    keyboardLayout?: string;
    uiLang?: string;
  } = $props();

  const hands = ['L', 'R'] as const;
  let activeFinger = $derived(fingerForKey(nextChar, keyboardLayout, isRussian));
  let fingerLabel = $derived(activeFinger
    ? `${t(uiLang, `hands.hand.${activeFinger[0]}`)} · ${t(uiLang, `hands.finger.${activeFinger[1]}`)}`
    : t(uiLang, 'hands.no_hint'));
</script>

<div class="hand-guide" role="group" aria-label={t(uiLang, 'hands.title')}>
  <div class="hands">
    {#each hands as hand}
      <div class="hand-column" class:selected={activeFinger.startsWith(hand)}>
        <svg class="hand" viewBox="0 0 200 240" role="img" aria-label={t(uiLang, `hands.hand.${hand}`)}>
          <title>{t(uiLang, `hands.hand.${hand}`)}</title>
          <g transform={hand === 'R' ? 'translate(200 0) scale(-1 1)' : undefined}>
            <path class="hand-surface" d={HAND_OUTLINE} />
            {#each HAND_FINGERS as finger}
              <g class="finger" class:active={activeFinger === `${hand}${finger.code}`} data-finger={`${hand}${finger.code}`}>
                <path class="finger-fill" d={`M${finger.start}${finger.contour}${finger.base}`} />
                <path class="finger-detail" d={finger.detail} />
              </g>
            {/each}
            <path class="hand-outline" d={HAND_OUTLINE} />
            <path class="wrist-line" d="M60 206Q101 217 141 207" />
          </g>
        </svg>
        <span class="hand-label"><span class="hand-dot" aria-hidden="true"></span>{t(uiLang, `hands.hand.${hand}`)}</span>
      </div>
    {/each}
  </div>
  <div class="finger-info">
    {#if nextChar}
      <kbd>{nextChar === ' ' ? t(uiLang, 'hands.space') : nextChar}</kbd>
      <span>{fingerLabel}</span>
    {:else}
      <span>{t(uiLang, 'hands.title')}</span>
    {/if}
  </div>
</div>

<style>
  .hand-guide { display:flex; flex-direction:column; align-items:center; gap:1rem; width:100%; max-width:32rem; padding:.5rem 1rem; }
  .hands { display:flex; justify-content:center; gap:clamp(1.5rem, 5vw, 3rem); width:100%; }
  .hand-column { display:flex; flex-direction:column; align-items:center; gap:.55rem; min-width:0; }
  .hand { display:block; width:clamp(7rem, 20vw, 10rem); max-width:100%; height:auto; overflow:visible; }
  .hand-surface { fill:var(--color-surface-raised, var(--bg-sub)); }
  .hand-outline { fill:none; stroke:var(--sub); stroke-width:1.6; stroke-linejoin:round; stroke-linecap:round; }
  .finger-fill { fill:transparent; }
  .finger.active .finger-fill { fill:var(--main); }
  .finger-detail, .wrist-line { fill:none; stroke:var(--sub); stroke-opacity:.35; stroke-width:1.4; stroke-linecap:round; }
  .finger.active .finger-detail { stroke:var(--bg); stroke-opacity:.55; }
  .hand-label { display:flex; align-items:center; gap:.45rem; color:var(--sub); font-size:.72rem; }
  .hand-dot { width:.35rem; height:.35rem; border-radius:50%; background:var(--sub); opacity:.4; }
  .selected .hand-label { color:var(--text); }
  .selected .hand-dot { background:var(--main); opacity:1; }
  .finger-info { display:flex; align-items:center; justify-content:center; flex-wrap:wrap; gap:.65rem; min-height:2rem; color:var(--sub); font-size:.75rem; text-align:center; }
  kbd { min-width:2rem; padding:.2rem .55rem; border:1px solid var(--color-border-strong, var(--sub)); border-radius:6px; background:var(--bg-sub); color:var(--main); font:inherit; font-weight:600; }
</style>
