<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/api/ipc';
  import { t } from '../lib/i18n';

  let { uiLang = 'en' }: { uiLang?: string } = $props();

  let achievements = $state<Array<{ id: string; name: string; description: string; unlocked: boolean }>>([]);
  let errorMsg = $state('');
  let filter = $state<'all' | 'unlocked' | 'locked'>('all');

  async function loadAchievements() {
    try {
      const data = await ipc.getAchievements() as any;
      achievements = Array.isArray(data) ? (data.length === 1 && Array.isArray(data[0]) ? data[0] : data) : [];
    } catch (e) {
      errorMsg = `Error: ${e}`;
    }
  }

  let filtered = $derived(
    filter === 'all' ? achievements
    : filter === 'unlocked' ? achievements.filter(a => a.unlocked)
    : achievements.filter(a => !a.unlocked)
  );

  let unlockedCount = $derived(achievements.filter(a => a.unlocked).length);

  onMount(loadAchievements);
</script>

<div class="gallery">
  <h2>{t(uiLang, 'nav.achievements')}</h2>

  {#if errorMsg}<p class="error">{errorMsg}</p>{/if}

  <div class="gallery-header">
    <span class="counter">{unlockedCount} / {achievements.length}</span>
    <div class="filters">
      <button class:active={filter === 'all'} onclick={() => filter = 'all'}>All</button>
      <button class:active={filter === 'unlocked'} onclick={() => filter = 'unlocked'}>Unlocked</button>
      <button class:active={filter === 'locked'} onclick={() => filter = 'locked'}>Locked</button>
    </div>
  </div>

  <div class="progress-bar">
    <div class="progress-fill" style="width: {achievements.length > 0 ? (unlockedCount / achievements.length) * 100 : 0}%"></div>
  </div>

  <div class="grid">
    {#each filtered as ach (ach.id)}
      <div class="card" class:locked={!ach.unlocked}>
        <div class="card-icon">{ach.unlocked ? '🏆' : '🔒'}</div>
        <div class="card-name">{ach.name}</div>
        <div class="card-desc">{ach.description}</div>
        {#if ach.unlocked}
          <div class="card-status unlocked">Unlocked</div>
        {:else}
          <div class="card-status locked">Locked</div>
        {/if}
      </div>
    {/each}
  </div>

  {#if filtered.length === 0}
    <p class="empty">
      {filter === 'all' ? t(uiLang, 'analytics.empty_achievements') : 'No achievements in this category.'}
    </p>
  {/if}
</div>

<style>
  .gallery { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  .gallery-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
  .counter { color: var(--main); font-size: 1rem; font-weight: bold; }
  .filters { display: flex; gap: 0.25rem; }
  .filters button {
    background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub);
    padding: 0.25rem 0.75rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px;
  }
  .filters button.active { color: var(--main); border-color: var(--main); }
  .progress-bar { height: 4px; background: var(--bg-sub); border-radius: 2px; margin-bottom: 1.5rem; overflow: hidden; }
  .progress-fill { height: 100%; background: var(--main); border-radius: 2px; transition: width 0.3s ease; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 0.75rem; }
  .card {
    display: flex; flex-direction: column; align-items: center; gap: 0.4rem;
    padding: 1.25rem 0.75rem; background: var(--bg-sub); border-radius: 8px;
    border: 1px solid transparent; transition: border-color 0.2s, transform 0.2s;
  }
  .card:hover { border-color: var(--main); transform: translateY(-2px); }
  .card.locked { opacity: 0.35; }
  .card-icon { font-size: 2rem; }
  .card-name { font-size: 0.85rem; color: var(--main); font-weight: bold; text-align: center; }
  .card-desc { font-size: 0.7rem; color: var(--sub); text-align: center; line-height: 1.3; }
  .card-status { font-size: 0.65rem; margin-top: 0.25rem; }
  .card-status.unlocked { color: #6c8; }
  .card-status.locked { color: var(--sub); }
  .empty { color: var(--sub); font-size: 0.875rem; text-align: center; margin-top: 2rem; }
  .error { color: var(--error); font-size: 0.875rem; }
</style>
