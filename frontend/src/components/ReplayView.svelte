<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/api/ipc';

  let testId = $state(0);
  let frames = $state<Array<{ frame_index: number; timestamp_ms: number; position: number; expected_char: string; typed_char: string | null; correct: boolean }>>([]);
  let currentFrame = $state(0);
  let playing = $state(false);
  let speed = $state(1);
  let playInterval: ReturnType<typeof setInterval> | null = null;

  async function loadReplay(id: number) {
    testId = id;
    try {
      const data = await ipc.getReplay(id) as unknown[][];
      frames = (data[0] as any[]) || [];
      currentFrame = 0;
    } catch {
      frames = [];
    }
  }

  function play() {
    playing = true;
    playInterval = setInterval(() => {
      if (currentFrame < frames.length - 1) {
        currentFrame++;
      } else {
        pause();
      }
    }, 100 / speed);
  }

  function pause() {
    playing = false;
    if (playInterval) { clearInterval(playInterval); playInterval = null; }
  }

  function seek(frame: number) {
    currentFrame = Math.max(0, Math.min(frame, frames.length - 1));
  }

  function setSpeed(s: number) {
    speed = s;
    if (playing) {
      pause();
      play();
    }
  }

  function getProgress(): number {
    if (frames.length === 0) return 0;
    return (currentFrame / frames.length) * 100;
  }

  function currentEvent() {
    return frames[currentFrame];
  }

  export { loadReplay };
</script>

{#if frames.length > 0}
  <div class="replay-view">
    <h3>Replay — Test #{testId}</h3>

    <div class="replay-text">
      {#each frames as f, i}
        <span
          class="replay-char {i < currentFrame ? (f.correct ? 'past-correct' : 'past-incorrect') : ''} {i === currentFrame ? 'current' : ''} {i > currentFrame ? 'future' : ''} {!f.correct && i <= currentFrame ? 'error' : ''}"
        >
          {f.expected_char === ' ' ? '\u00A0' : f.expected_char}
        </span>
      {/each}
    </div>

    <div class="replay-info">
      {#if currentEvent()}
        <span>Frame: {currentFrame}/{frames.length}</span>
        <span>Key: {currentEvent()?.typed_char || '—'}</span>
        <span>{currentEvent()?.correct ? '✓' : '✗'}</span>
        <span>Time: {currentEvent()?.timestamp_ms}ms</span>
      {/if}
    </div>

    <div class="replay-progress-bar">
      <div class="progress-fill" style="width: {getProgress()}%"></div>
    </div>

    <div class="replay-controls">
      <button onclick={() => seek(0)}>⏮</button>
      {#if playing}
        <button onclick={pause}>⏸</button>
      {:else}
        <button onclick={play}>▶</button>
      {/if}
      <button onclick={() => seek(frames.length - 1)}>⏭</button>
      <div class="speed-selector">
        {#each [0.5, 1, 2, 4] as s}
          <button class:active={speed === s} onclick={() => setSpeed(s)}>{s}x</button>
        {/each}
      </div>
    </div>
  </div>
{:else}
  <p class="empty">No replay data for test #{testId}</p>
{/if}

<style>
  .replay-view { max-width: 900px; width: 100%; }
  h3 { color: var(--main); font-size: 1.1rem; margin-bottom: 0.5rem; }
  .replay-text { font-size: 1.5rem; line-height: 1.8; padding: 1.5rem; background: var(--bg-sub); border-radius: 8px; text-align: center; }
  .replay-char { position: relative; transition: color 0.05s; }
  .replay-char.past-correct { color: var(--text); opacity: 0.5; }
  .replay-char.past-incorrect { color: var(--error); opacity: 0.5; }
  .replay-char.current { color: var(--main); font-weight: bold; }
  .replay-char.future { color: var(--sub); }
  .replay-char.error { color: var(--error); }
  .replay-info { display: flex; gap: 1.5rem; font-size: 0.875rem; color: var(--sub); margin: 0.5rem 0; justify-content: center; }
  .replay-progress-bar { height: 4px; background: var(--bg-sub); border-radius: 2px; margin: 0.5rem 0; }
  .progress-fill { height: 100%; background: var(--main); border-radius: 2px; transition: width 0.1s; }
  .replay-controls { display: flex; gap: 0.5rem; align-items: center; justify-content: center; }
  .replay-controls button { background: var(--bg-sub); color: var(--text); border: 1px solid var(--sub); padding: 0.5rem 1rem; font-family: inherit; font-size: 0.875rem; cursor: pointer; border-radius: 4px; }
  .replay-controls button:hover { border-color: var(--main); color: var(--main); }
  .speed-selector { display: flex; gap: 0.25rem; margin-left: 0.5rem; }
  .speed-selector button { padding: 0.25rem 0.5rem; font-size: 0.75rem; }
  .speed-selector button.active { color: var(--main); border-color: var(--main); }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
</style>