<script lang="ts">
  import type { ModuleResponse, LessonResponse } from '../types/index';

  let {
    modules,
    progress,
    language,
    onSelectLesson,
  }: {
    modules: ModuleResponse[];
    progress: Record<string, { status: string; best_wpm: number; best_accuracy: number }>;
    language: string;
    onSelectLesson: (lessonId: string, language: string) => void;
  } = $props();

  function getStatus(lessonId: string): string {
    return progress[lessonId]?.status || 'not_started';
  }

  function getBestWpm(lessonId: string): number {
    return progress[lessonId]?.best_wpm || 0;
  }
</script>

<div class="list-view">
  <h2>Course — {language === 'en' ? 'English' : 'Русский'}</h2>
  {#each modules as m}
    <div class="module">
      <h3>{m.name} <span class="difficulty">{m.difficulty}</span></h3>
      <div class="lessons">
        {#each m.lessons as l}
          <button
            class="lesson-card {getStatus(l.id)}"
            onclick={() => onSelectLesson(l.id, language)}
          >
            <span class="lesson-name">{l.name}</span>
            <span class="lesson-status">
              {#if getStatus(l.id) === 'completed'}✓{/if}
              {#if getBestWpm(l.id) > 0}<span class="lesson-wpm">{getBestWpm(l.id).toFixed(0)} WPM</span>{/if}
            </span>
          </button>
        {/each}
      </div>
    </div>
  {/each}
</div>

<style>
  .list-view { max-width: 900px; width: 100%; }
  h2 { color: var(--main); font-size: 1.5rem; margin-bottom: 1rem; }
  .module { margin-bottom: 1.5rem; }
  h3 { color: var(--main); font-size: 1.1rem; margin: 0 0 0.5rem; }
  .difficulty { font-size: 0.75rem; color: var(--sub); text-transform: uppercase; }
  .lessons { display: flex; flex-direction: column; gap: 0.25rem; }
  .lesson-card {
    display: flex; justify-content: space-between; align-items: center;
    background: var(--bg-sub); border: 1px solid var(--sub); border-radius: 4px;
    padding: 0.5rem 1rem; cursor: pointer; font-family: inherit; font-size: 0.875rem;
    color: var(--text);
  }
  .lesson-card.completed { border-color: var(--main); }
  .lesson-card.in_progress { border-color: var(--text); }
  .lesson-card:hover { background: var(--bg); }
  .lesson-status { display: flex; gap: 0.5rem; align-items: center; }
  .lesson-wpm { font-size: 0.75rem; color: var(--sub); }
</style>