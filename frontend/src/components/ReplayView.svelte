<script lang="ts">
  import { t } from '../lib/i18n';
  import type { ReplayFrame } from '../lib/types/index';

  type ReplaySpeed = 0.5 | 1 | 2 | 4;
  type ReplayChar = {
    expected: string;
    typed: string | null;
    status: 'pending' | 'correct' | 'incorrect';
  };

  let {
    testId,
    frames,
    onClose,
    uiLang = 'en',
  }: {
    testId: number;
    frames: ReplayFrame[];
    onClose: () => void;
    uiLang?: string;
  } = $props();

  let currentFrameIndex = $state(0);
  let playing = $state(false);
  let speed = $state<ReplaySpeed>(1);

  let orderedFrames = $derived(
    [...frames].sort((left, right) => left.frame_index - right.frame_index),
  );
  let currentFrame = $derived(orderedFrames[currentFrameIndex] ?? null);
  let replayChars = $derived(buildReplayChars(orderedFrames, currentFrameIndex));
  let progress = $derived(
    orderedFrames.length <= 1
      ? (orderedFrames.length === 1 ? 100 : 0)
      : (currentFrameIndex / (orderedFrames.length - 1)) * 100,
  );

  $effect(() => {
    testId;
    frames.length;
    currentFrameIndex = 0;
    playing = false;
  });

  $effect(() => {
    const isPlaying = playing;
    const playbackSpeed = speed;
    const timeline = orderedFrames;
    const frameCount = timeline.length;
    if (!isPlaying || frameCount <= 1) return;

    let elapsedSinceAdvance = 0;
    let lastTick = performance.now();
    const interval = window.setInterval(() => {
      const now = performance.now();
      elapsedSinceAdvance += (now - lastTick) * playbackSpeed;
      lastTick = now;

      while (currentFrameIndex < frameCount - 1) {
        const delayMs = Math.max(
          0,
          timeline[currentFrameIndex + 1].timestamp_ms - timeline[currentFrameIndex].timestamp_ms,
        );
        if (elapsedSinceAdvance < delayMs) break;

        elapsedSinceAdvance -= delayMs;
        currentFrameIndex += 1;
      }

      if (currentFrameIndex >= frameCount - 1) playing = false;
    }, 16);

    return () => window.clearInterval(interval);
  });

  function affectedPosition(frame: ReplayFrame): number {
    return frame.correct ? Math.max(0, frame.position - 1) : Math.max(0, frame.position);
  }

  function buildReplayChars(allFrames: ReplayFrame[], throughIndex: number): ReplayChar[] {
    if (allFrames.length === 0) return [];

    const maxPosition = allFrames.reduce(
      (maximum, frame) => Math.max(maximum, affectedPosition(frame)),
      0,
    );
    const chars: ReplayChar[] = Array.from({ length: maxPosition + 1 }, () => ({
      expected: '·',
      typed: null,
      status: 'pending',
    }));

    for (const frame of allFrames) {
      chars[affectedPosition(frame)].expected = frame.expected_char;
    }

    for (let index = 0; index <= throughIndex && index < allFrames.length; index += 1) {
      const frame = allFrames[index];
      const position = affectedPosition(frame);
      if (frame.typed_char === 'Backspace') {
        chars[position] = { ...chars[position], typed: null, status: 'pending' };
      } else if (frame.correct) {
        chars[position] = {
          ...chars[position],
          typed: frame.typed_char,
          status: 'correct',
        };
      } else {
        chars[position] = {
          ...chars[position],
          typed: frame.typed_char,
          status: 'incorrect',
        };
      }
    }

    return chars;
  }

  function togglePlayback() {
    if (orderedFrames.length <= 1) return;
    if (!playing && currentFrameIndex >= orderedFrames.length - 1) currentFrameIndex = 0;
    playing = !playing;
  }

  function seek(frameIndex: number) {
    currentFrameIndex = Math.max(0, Math.min(frameIndex, orderedFrames.length - 1));
    if (currentFrameIndex >= orderedFrames.length - 1) playing = false;
  }

  function setSpeed(nextSpeed: ReplaySpeed) {
    speed = nextSpeed;
  }
</script>

<div class="replay-backdrop" role="presentation">
  <div class="replay-dialog" role="dialog" aria-modal="true" aria-labelledby="replay-title">
    <header>
      <h3 id="replay-title">{t(uiLang, 'replay.title')} #{testId}</h3>
      <button class="close-button" type="button" aria-label={t(uiLang, 'replay.close')} onclick={onClose}>×</button>
    </header>

    {#if orderedFrames.length === 0}
      <p class="empty">{t(uiLang, 'replay.empty')}</p>
    {:else}
      <div class="replay-text" aria-live="polite">
        {#each replayChars as character, index}
          <span
            class="replay-char {character.status}"
            class:caret={currentFrame?.position === index}
            title={character.typed ?? character.expected}
          >
            {character.expected === ' ' ? '\u00A0' : character.expected}
          </span>
        {/each}
        {#if currentFrame && currentFrame.position >= replayChars.length}
          <span class="end-caret" aria-hidden="true"></span>
        {/if}
      </div>

      <div class="frame-details">
        <span>{t(uiLang, 'replay.frame')}: {currentFrameIndex + 1}/{orderedFrames.length}</span>
        <span>{t(uiLang, 'replay.key')}: {currentFrame?.typed_char ?? '—'}</span>
        <span>{t(uiLang, 'replay.expected')}: {currentFrame?.expected_char ?? '—'}</span>
        <span>{t(uiLang, 'replay.position')}: {currentFrame?.position ?? 0}</span>
        <span>{((currentFrame?.timestamp_ms ?? 0) / 1000).toFixed(2)}s</span>
        <span class:correct-result={currentFrame?.correct} class:error-result={!currentFrame?.correct}>
          {currentFrame?.correct ? '✓' : '✗'}
        </span>
      </div>

      <div class="progress-track" aria-hidden="true">
        <div class="progress-fill" style:width={`${progress}%`}></div>
      </div>
      <input
        class="seek-slider"
        type="range"
        min="0"
        max={Math.max(0, orderedFrames.length - 1)}
        value={currentFrameIndex}
        aria-label={t(uiLang, 'replay.seek')}
        oninput={(event) => seek(Number(event.currentTarget.value))}
      />

      <div class="replay-controls">
        <button type="button" onclick={() => seek(0)} disabled={currentFrameIndex === 0} aria-label={t(uiLang, 'replay.first')}>⏮</button>
        <button type="button" onclick={togglePlayback} disabled={orderedFrames.length <= 1}>
          {playing ? `⏸ ${t(uiLang, 'replay.pause')}` : `▶ ${t(uiLang, 'replay.play')}`}
        </button>
        <button type="button" onclick={() => seek(orderedFrames.length - 1)} disabled={currentFrameIndex >= orderedFrames.length - 1} aria-label={t(uiLang, 'replay.last')}>⏭</button>
        <div class="speed-selector" aria-label={t(uiLang, 'replay.speed')}>
          {#each [0.5, 1, 2, 4] as option}
            <button
              type="button"
              class:active={speed === option}
              aria-pressed={speed === option}
              onclick={() => setSpeed(option as ReplaySpeed)}
            >{option}×</button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .replay-backdrop {
    position: fixed; inset: 0; z-index: 200; display: grid; place-items: center;
    padding: 1rem; background: rgba(0, 0, 0, 0.65);
  }
  .replay-dialog {
    width: min(900px, 100%); max-height: calc(100vh - 2rem); overflow: auto;
    padding: 1.25rem; border: 1px solid var(--sub); border-radius: 8px;
    background: var(--bg); color: var(--text);
  }
  header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  h3 { margin: 0; color: var(--main); font-size: 1.1rem; }
  button {
    border: 1px solid var(--sub); border-radius: 4px; padding: 0.4rem 0.75rem;
    background: var(--bg-sub); color: var(--text); cursor: pointer; font: inherit;
    font-size: 0.75rem;
  }
  button:hover:not(:disabled), button.active { border-color: var(--main); color: var(--main); }
  button:disabled { cursor: not-allowed; opacity: 0.4; }
  .close-button { padding: 0.2rem 0.55rem; font-size: 1.2rem; }
  .empty { padding: 2rem; color: var(--sub); text-align: center; }
  .replay-text {
    display: flex; flex-wrap: wrap; justify-content: center; min-height: 4em;
    padding: 1.5rem; border-radius: 8px; background: var(--bg-sub);
    font-size: 1.5rem; line-height: 1.8;
  }
  .replay-char { position: relative; color: var(--sub); opacity: 0.55; }
  .replay-char.correct { color: var(--text); opacity: 1; }
  .replay-char.incorrect { color: var(--error); opacity: 1; }
  .replay-char.caret::before, .end-caret::before {
    content: ''; position: absolute; top: 0.15em; bottom: 0.15em; left: -0.08em;
    border-left: 0.08em solid var(--caret); animation: blink 1s infinite;
  }
  .end-caret { position: relative; width: 0; }
  .frame-details {
    display: flex; flex-wrap: wrap; justify-content: center; gap: 0.5rem 1.25rem;
    margin: 0.75rem 0; color: var(--sub); font-size: 0.75rem;
  }
  .correct-result { color: #6c8; }
  .error-result { color: var(--error); }
  .progress-track { height: 4px; overflow: hidden; border-radius: 2px; background: var(--bg-sub); }
  .progress-fill { height: 100%; background: var(--main); transition: width 0.08s linear; }
  .seek-slider { width: 100%; accent-color: var(--main); }
  .replay-controls { display: flex; flex-wrap: wrap; justify-content: center; align-items: center; gap: 0.5rem; }
  .speed-selector { display: flex; gap: 0.25rem; margin-left: 0.5rem; }
  .speed-selector button { padding: 0.3rem 0.5rem; }
  @keyframes blink { 0%, 50% { opacity: 1; } 51%, 100% { opacity: 0; } }
</style>
