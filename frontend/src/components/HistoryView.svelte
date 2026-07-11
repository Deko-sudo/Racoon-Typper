<script lang="ts">
  import * as ipc from '../lib/api/ipc';
  import { t } from '../lib/i18n';
  import type { ReplayFrame, TestSummary } from '../lib/types/index';
  import ReplayView from './ReplayView.svelte';

  let { history, total, uiLang = 'en' }: { history: TestSummary[]; total: number; uiLang?: string } = $props();
  let replayAvailability = $state<Record<number, boolean>>({});
  let replayLoadingId = $state<number | null>(null);
  let selectedReplay = $state<{ testId: number; frames: ReplayFrame[] } | null>(null);
  let replayError = $state('');

  $effect(() => {
    const testIds = history.map((test) => test.id);
    let cancelled = false;
    replayAvailability = {};

    const checks = testIds.map(async (testId) => {
      try {
        return [testId, await ipc.hasReplay(testId)] as const;
      } catch {
        return [testId, false] as const;
      }
    });
    void Promise.all(checks).then((entries) => {
      if (!cancelled) replayAvailability = Object.fromEntries(entries);
    });

    return () => { cancelled = true; };
  });

  function formatDate(iso: string): string {
    try { return new Date(iso).toLocaleString(); } catch { return iso; }
  }

  async function openReplay(testId: number) {
    replayLoadingId = testId;
    replayError = '';
    try {
      const frames = await ipc.getReplay(testId);
      selectedReplay = { testId, frames };
    } catch (error) {
      replayError = `${t(uiLang, 'history.replay_error')}: ${error}`;
    } finally {
      replayLoadingId = null;
    }
  }
</script>

<div class="list-view">
  <h2>{t(uiLang, 'history.title')} ({total})</h2>
  {#if history.length === 0}
    <p class="empty">{t(uiLang, 'history.empty')}</p>
  {:else}
    <table>
      <thead><tr><th>{t(uiLang, 'history.date')}</th><th>{t(uiLang, 'history.mode')}</th><th>{t(uiLang, 'history.wpm')}</th><th>{t(uiLang, 'history.accuracy')}</th><th>{t(uiLang, 'history.duration')}</th><th>{t(uiLang, 'history.pb')}</th><th>{t(uiLang, 'history.actions')}</th></tr></thead>
      <tbody>
        {#each history as h}
          <tr>
            <td>{formatDate(h.created_at)}</td>
            <td>{h.mode_type}</td>
            <td>{h.wpm.toFixed(1)}</td>
            <td>{h.accuracy.toFixed(1)}%</td>
            <td>{(h.duration_ms / 1000).toFixed(1)}s</td>
            <td>{h.is_pb ? '★' : ''}</td>
            <td>
              <button
                type="button"
                disabled={replayAvailability[h.id] !== true || replayLoadingId !== null}
                onclick={() => openReplay(h.id)}
              >
                {#if replayLoadingId === h.id}
                  {t(uiLang, 'history.loading_replay')}
                {:else if replayAvailability[h.id] === true}
                  {t(uiLang, 'history.watch_replay')}
                {:else if replayAvailability[h.id] === false}
                  {t(uiLang, 'history.no_replay')}
                {:else}
                  …
                {/if}
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
  {#if replayError}<p class="replay-error" role="alert">{replayError}</p>{/if}
</div>

{#if selectedReplay}
  <ReplayView
    testId={selectedReplay.testId}
    frames={selectedReplay.frames}
    onClose={() => { selectedReplay = null; }}
    {uiLang}
  />
{/if}

<style>
  .list-view { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  .empty { color: var(--sub); text-align: center; padding: 2rem; }
  table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
  th { text-align: left; color: var(--main); padding: 0.5rem; border-bottom: 1px solid var(--bg-sub); }
  td { padding: 0.5rem; color: var(--text); border-bottom: 1px solid var(--bg-sub); }
  td button {
    border: 1px solid var(--sub); border-radius: 4px; padding: 0.3rem 0.6rem;
    background: var(--bg-sub); color: var(--text); cursor: pointer; font: inherit; font-size: 0.65rem;
  }
  td button:hover:not(:disabled) { border-color: var(--main); color: var(--main); }
  td button:disabled { cursor: not-allowed; opacity: 0.45; }
  .replay-error { margin-top: 0.75rem; color: var(--error); font-size: 0.75rem; }
</style>
