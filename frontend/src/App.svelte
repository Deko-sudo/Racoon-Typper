<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from './lib/api/ipc';
  import { t } from './lib/i18n';
  import { createNavigationStore } from './lib/stores/navigation.svelte';
  import { createNotificationStore } from './lib/stores/notifications.svelte';
  import { createTestSessionStore } from './lib/stores/testSession.svelte';
  import { createSettingsStore } from './lib/stores/settings.svelte';
  import { createContentStore } from './lib/stores/content.svelte';
  import { vimActionForKey, findMatches, VIM_VIEWS } from './lib/vimNavigation';
  import type { ViewName } from './lib/types/index';

  import NavigationBar from './components/NavigationBar.svelte';
  import TestView from './components/TestView.svelte';
  import HistoryView from './components/HistoryView.svelte';
  import BestsView from './components/BestsView.svelte';
  import CustomTextsView from './components/CustomTextsView.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import LessonListView from './components/LessonListView.svelte';
  import WeakKeysPanel from './components/WeakKeysPanel.svelte';
  import TypingWarnings from './components/TypingWarnings.svelte';
  import NotificationStack from './components/NotificationStack.svelte';
  import DashboardView from './components/DashboardView.svelte';
  import AnalyticsView from './components/AnalyticsView.svelte';
  import AchievementGallery from './components/AchievementGallery.svelte';
  import PomodoroView from './components/PomodoroView.svelte';
  import CheatsheetOverlay from './components/CheatsheetOverlay.svelte';
  import VimSearchOverlay from './components/VimSearchOverlay.svelte';
  import OnboardingView from './components/OnboardingView.svelte';
  import type { OnboardingResult } from './lib/onboarding';

  // Navigation
  const navigation = createNavigationStore('test');
  const view = $derived(navigation.view);

  // Notifications
  const notificationStore = createNotificationStore();

  // Global presentation state
  let errorMsg = $state('');
  // Zen mode — hide everything except text
  let zenActive = $state(false);
  // Cheatsheet overlay
  let cheatsheetOpen = $state(false);
  // Vim '/' search: visual highlight only — the backend caret is never moved.
  let vimSearchOpen = $state(false);
  let searchMatches = $state(new Set<number>());
  let searchMatchCount = $state(0);
  // Tracks a pending single 'g' press for the 'gg' Vim scroll-to-top command.
  let vimPendingG = $state(false);
  // When the pending 'g' was pressed (for the 1s 'gg' expiry).
  let vimPendingGAt = 0;
  // Achievement tracking — snapshot before test for auto-toast
  let preTestAchievements = $state<Array<{ id: string; unlocked: boolean }>>([]);
  let audioContext: AudioContext | null = null;
  // First-run onboarding gate: shown once until completed or skipped.
  let showOnboarding = $state(false);

  // Feature stores
  const settingsStore = createSettingsStore({
    setError: (message) => { errorMsg = message; },
  });
  const contentStore = createContentStore({
    setError: (message) => { errorMsg = message; },
  });
  const testSession = createTestSessionStore({
    getSettings: () => settingsStore.settings,
    getUiLang: () => settingsStore.uiLang,
    setError: (message) => { errorMsg = message; },
    setZenActive: (active) => { zenActive = active; },
    playSound: (event) => void playSound(event),
    notify: (type, message) => notificationStore.add(type, message),
    beforeStart: snapshotAchievements,
    getCurrentLessonId: () => contentStore.currentLessonId,
    setCurrentLessonId: (lessonId) => { contentStore.currentLessonId = lessonId; },
    onLessonCompleted: (lessonId, stats) => {
      // The backend persisted lesson completion in the same transaction as the
      // completed test, applying the pass gate (accuracy ≥90% AND wpm ≥20).
      // Mirror that gate locally so the UI matches the persisted status.
      const passed = stats.accuracy >= 90 && stats.wpm >= 20;
      contentStore.applyLessonCompletion(lessonId, passed, stats.wpm, stats.accuracy);
    },
    onHistoryChanged: () => void contentStore.loadHistoryTotal(),
    onAchievementsChanged: checkNewAchievements,
    onStarted: () => switchView('test'),
  });

  const settings = $derived(settingsStore.settings);
  const uiLang = $derived(settingsStore.uiLang);
  const themes = $derived(settingsStore.themes);
  const activeTheme = $derived(settingsStore.activeTheme);
  const mainFontSize = $derived(settingsStore.mainFontSize);

  async function snapshotAchievements() {
    try {
      const achievements = (await ipc.getAchievements()).flat();
      preTestAchievements = achievements.map((achievement) => ({
        id: achievement.id,
        unlocked: achievement.unlocked,
      }));
    } catch {
      preTestAchievements = [];
    }
  }

  async function checkNewAchievements() {
    try {
      const after = (await ipc.getAchievements()).flat();
      for (const a of after) {
        if (a.unlocked && !preTestAchievements.find(p => p.id === a.id && p.unlocked)) {
          notificationStore.add('SUCCESS', `${a.name} — ${a.description}`);
        }
      }
    } catch {
      // ignore
    }
  }

  async function playSound(event: 'key_press' | 'error' | 'lesson_complete') {
    if (!settings?.sound_enabled || settings.sound_volume <= 0) return;
    try {
      audioContext ??= new AudioContext();
      if (audioContext.state === 'suspended') await audioContext.resume();
      const sound = await ipc.getSoundEvent(event);
      if (!sound) return;

      const oscillator = audioContext.createOscillator();
      const gain = audioContext.createGain();
      oscillator.frequency.value = sound.frequency;
      gain.gain.value = Math.min(sound.volume, settings.sound_volume);
      oscillator.connect(gain);
      gain.connect(audioContext.destination);
      oscillator.onended = () => {
        oscillator.disconnect();
        gain.disconnect();
      };
      oscillator.start();
      oscillator.stop(audioContext.currentTime + sound.duration_ms / 1000);
    } catch (error) {
      console.warn('Sound playback failed:', error);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    const activeElement = document.activeElement;
    if (
      activeElement instanceof HTMLInputElement
      || activeElement instanceof HTMLTextAreaElement
      || (activeElement instanceof HTMLElement && activeElement.isContentEditable)
    ) {
      return;
    }

    // Cheatsheet overlay: '?' toggles, Esc closes. Handled before vim-mode so
    // the overlay never leaks keys into navigation or a running test.
    if (cheatsheetOpen) {
      if (e.key === 'Escape' || e.key === '?') {
        e.preventDefault();
        cheatsheetOpen = false;
      }
      return;
    }
    if (e.key === '?' && !testSession.isRunning) {
      e.preventDefault();
      cheatsheetOpen = true;
      return;
    }

    // Vim '/' search: opens the search bar; Esc closes it. Visual only.
    if (vimSearchOpen) {
      if (e.key === 'Escape') {
        e.preventDefault();
        vimSearchOpen = false;
        searchMatches = new Set();
        searchMatchCount = 0;
      }
      return;
    }
    if (e.key === '/' && settings?.vim_mode && !testSession.isRunning) {
      e.preventDefault();
      vimSearchOpen = true;
      return;
    }

    // Vim mode navigation (only when not actively typing a test)
    if (settings?.vim_mode && !testSession.isRunning) {
      // 'gg' — это двойное нажатие: одиночный 'g' истекает через 1 секунду,
      // иначе залипший pending-g срабатывает на первом 'g' следующего теста.
      if (vimPendingG && Date.now() - vimPendingGAt > 1000) vimPendingG = false;
      const { action, nextPendingG } = vimActionForKey(e.key, view, vimPendingG);
      vimPendingG = nextPendingG;
      if (vimPendingG) vimPendingGAt = Date.now();
      switch (action.type) {
        case 'prev_tab': {
          const idx = VIM_VIEWS.indexOf(view as (typeof VIM_VIEWS)[number]);
          if (idx > 0) { e.preventDefault(); switchView(VIM_VIEWS[idx - 1]); }
          return;
        }
        case 'next_tab': {
          const idx = VIM_VIEWS.indexOf(view as (typeof VIM_VIEWS)[number]);
          if (idx >= 0 && idx < VIM_VIEWS.length - 1) { e.preventDefault(); switchView(VIM_VIEWS[idx + 1]); }
          return;
        }
        case 'scroll_up': e.preventDefault(); window.scrollBy(0, -100); return;
        case 'scroll_down': e.preventDefault(); window.scrollBy(0, 100); return;
        case 'scroll_top': e.preventDefault(); window.scrollTo(0, 0); return;
        case 'scroll_bottom': e.preventDefault(); window.scrollTo(0, document.body.scrollHeight); return;
        case 'restart': e.preventDefault(); void testSession.restartTest(); return;
        case 'none': return;
      }
    }

    if (!testSession.isRunning || testSession.isComplete) return;

    // View-gating: клавиши попадают в тест только на тестовых вью.
    // WeakKeys нужна для inline-training, остальные вью не трогают сессию.
    if (view !== 'test' && view !== 'weakkeys') return;

    // IME-фильтр (ja/zh/ko): во время composition keydown приходит с
    // isComposing/keyCode 229 — пропускать, иначе мусор попадает в тест.
    if (e.isComposing || e.keyCode === 229) return;

    // Caps Lock detection
    if (e.getModifierState && e.getModifierState('CapsLock') !== testSession.capsLockOn) {
      testSession.capsLockOn = e.getModifierState('CapsLock');
      if (testSession.capsLockOn && settings?.show_capslock_warnings) {
        notificationStore.add('WARNING', t(uiLang, 'warning.caps_title'));
      }
    }

    // Модификатор-комбо (Ctrl+C/V, Alt+...) — не печатные символы.
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;

    // Whitelist: только печатные символы (key.length === 1) и Backspace.
    // Раньше сюда просачивались Arrow/Delete/Home/F-клавиши, и бэкенд
    // превращал первую букву имени клавиши в фантомный символ ('A', 'D', 'H').
    if (e.key.length !== 1 && e.key !== 'Backspace') return;

    e.preventDefault();

    // Track last typed char for layout detection
    if (e.key.length === 1) {
      testSession.lastTypedChar = e.key;
      testSession.testStartedAt ??= Date.now();
    }
    testSession.enqueueKey(e.key, e.code);
  }

  $effect(() => {
    const startedAt = testSession.testStartedAt;
    const running = testSession.isRunning;
    const complete = testSession.isComplete;
    const mode = testSession.sessionModeType;
    const durationMs = testSession.sessionDurationMs;
    if (!running || complete || startedAt === null) return;

    const updateClock = () => {
      const currentElapsed = Math.max(0, Date.now() - startedAt);
      testSession.elapsedMs = mode === 'time' && durationMs > 0
        ? Math.min(currentElapsed, durationMs)
        : currentElapsed;
      if (
        mode === 'time'
        && durationMs > 0
        && currentElapsed >= durationMs
        && !testSession.timeCompletionQueued
      ) {
        testSession.timeCompletionQueued = true;
        testSession.enqueueKey('', '', true);
      }
    };

    updateClock();
    const interval = window.setInterval(updateClock, 50);
    return () => window.clearInterval(interval);
  });

  function switchView(v: ViewName) {
    navigation.navigate(v);
    // Уход с тестовых вью на любой другой — абандоним бегущую сессию,
    // иначе таймер доедет в фоне и запишет брошенный тест в историю
    // (и пометит урок выполненным).
    if (v !== 'test' && v !== 'weakkeys' && testSession.isRunning && !testSession.isComplete) {
      void testSession.abandonActiveSessionForReplacement();
    }
    if (v === 'history') contentStore.loadHistory();
    if (v === 'bests') contentStore.loadBests();
    if (v === 'custom') contentStore.loadCustomTexts();
    if (v === 'lessons') contentStore.loadLessons();
    if (v === 'weakkeys') contentStore.loadWeakKeys();
    if (v === 'dashboard') contentStore.loadDashboard();
  }

  function applyVimSearch(query: string) {
    const matches = findMatches(testSession.text, query);
    searchMatches = matches;
    searchMatchCount = matches.size > 0 ? countMatches(matches) : 0;
  }

  function countMatches(matches: Set<number>): number {
    // Считаем непрерывные диапазоны позиций как отдельные совпадения.
    const sorted = [...matches].sort((a, b) => a - b);
    let count = 0;
    let previous = -2;
    for (const position of sorted) {
      if (position !== previous + 1) count += 1;
      previous = position;
    }
    return count;
  }

  function closeVimSearch() {
    vimSearchOpen = false;
    searchMatches = new Set();
    searchMatchCount = 0;
  }

  function onRepeatLesson() {
    if (!contentStore.currentLessonId) return;
    void testSession.startLesson(contentStore.currentLessonId, contentStore.lessonLang);
  }

  function onNextLesson() {
    const next = contentStore.lessonNavigation?.nextLessonId;
    if (!next) return;
    void testSession.startLesson(next, contentStore.lessonLang);
  }

  function onReturnToLessons() {
    contentStore.currentLessonId = null;
    switchView('lessons');
  }

  onMount(async () => {
    try {
      await settingsStore.loadThemes();
    } catch (error) {
      errorMsg = `Theme catalog error: ${ipc.ipcErrorMessage(error)}`;
    }
    try {
      await settingsStore.loadSettings();
    } catch (error) {
      errorMsg = `Settings load error: ${ipc.ipcErrorMessage(error)}`;
    }
    // A window/hot reload restarts the renderer but the in-memory backend
    // engine keeps any prior session, so abandon it before starting fresh.
    // Safe: engine.abort() only discards a Running session.
    try {
      await ipc.abandonActiveSession();
    } catch {
      // Non-fatal — startTest below will surface a real lifecycle error.
    }
    if (settingsStore.settings && !settingsStore.settings.onboarding_completed) {
      showOnboarding = true;
      return;
    }
    await testSession.startTest();
  });

  function applyOnboardingLanguage(result: OnboardingResult) {
    testSession.selectedLanguage = result.practiceLanguage;
    contentStore.lessonLang = result.practiceLanguage;
  }

  async function persistOnboardingResult(result: OnboardingResult) {
    await settingsStore.updateSetting('practice_language', result.practiceLanguage);
    await settingsStore.updateSetting('daily_goal_type', result.goalType);
    if (result.goalType === 'time') {
      await settingsStore.updateSetting('daily_goal_minutes', result.goalMinutes);
    } else if (result.goalType === 'wpm') {
      await settingsStore.updateSetting('daily_goal_wpm', result.goalWpm);
    } else {
      await settingsStore.updateSetting('daily_goal_accuracy', result.goalAccuracy);
    }
    await settingsStore.updateSetting('onboarding_completed', true);
  }

  async function handleOnboardingComplete(result: OnboardingResult) {
    showOnboarding = false;
    try {
      await persistOnboardingResult(result);
    } catch (error) {
      errorMsg = `Onboarding save error: ${ipc.ipcErrorMessage(error)}`;
    }
    applyOnboardingLanguage(result);
    await testSession.startTest();
  }

  async function handleOnboardingSkip() {
    showOnboarding = false;
    try {
      await settingsStore.updateSetting('onboarding_completed', true);
    } catch (error) {
      errorMsg = `Onboarding save error: ${ipc.ipcErrorMessage(error)}`;
    }
    await testSession.startTest();
  }

  function handleOnboardingUiLang(lang: string) {
    void settingsStore.updateSetting('ui_language', lang);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main style:font-size={mainFontSize} data-session-state={testSession.sessionState}>
  {#if !zenActive}
    <NavigationBar {view} historyTotal={contentStore.historyTotal} {uiLang} onNavigate={switchView} />
  {/if}

  {#if settings?.vim_mode && !testSession.isRunning}
    <div class="vim-indicator" aria-label="Vim mode active">VIM</div>
  {/if}

  {#if errorMsg}
    <p class="error">{errorMsg}</p>
  {/if}

  {#if view === 'dashboard'}
    <DashboardView
      stats={contentStore.dashboardStats}
      weekly={contentStore.weeklySummaries}
      onNavigate={(v) => switchView(v as ViewName)}
      weakKeys={contentStore.weakKeysData}
      onStartTraining={() => void testSession.startTraining()}
      uiLang={uiLang}
    />
  {:else if view === 'test'}
    {#if testSession.isRunning}
      <TypingWarnings
        expectedLanguage={testSession.sessionLanguage}
        lastTypedChar={testSession.lastTypedChar}
        capsLockOn={testSession.capsLockOn}
        showLayoutWarnings={settings?.show_layout_warnings ?? true}
        showCapsLockWarnings={settings?.show_capslock_warnings ?? true}
        {uiLang}
      />
    {/if}
    <TestView
      text={testSession.text}
      caretPos={testSession.caretPos}
      charStatuses={testSession.charStatuses}
      erroredPositions={testSession.erroredPositions}
      {searchMatches}
      isRunning={testSession.isRunning}
      isComplete={testSession.isComplete}
      liveWpm={testSession.liveWpm}
      liveAccuracy={testSession.liveAccuracy}
      elapsedMs={testSession.elapsedMs}
      finalStats={testSession.finalStats}
      {settings}
      selectedMode={testSession.selectedMode}
      selectedDuration={testSession.selectedDuration}
      selectedWordCount={testSession.selectedWordCount}
      selectedLanguage={testSession.selectedLanguage}
      sessionModeType={testSession.sessionModeType}
      sessionLanguage={testSession.sessionLanguage}
      onModeChange={testSession.onModeChange}
      onDurationChange={testSession.onDurationChange}
      onWordCountChange={testSession.onWordCountChange}
      onLanguageChange={testSession.onLanguageChange}
      onAbort={() => void testSession.abortTest()}
      onRestart={() => void testSession.restartTest()}
      lessonNavigation={contentStore.lessonNavigation}
      {onRepeatLesson}
      {onNextLesson}
      {onReturnToLessons}
      uiLang={uiLang}
    />
  {:else if view === 'pomodoro'}
    <PomodoroView
      {settings}
      {uiLang}
      onUpdateSetting={settingsStore.updateSetting}
      onPhaseComplete={() => void playSound('lesson_complete')}
    />
  {:else if view === 'history'}
    <HistoryView
      history={contentStore.history}
      total={contentStore.historyTotal}
      page={contentStore.historyPage}
      pageSize={20}
      onPrevPage={contentStore.historyPrevPage}
      onNextPage={contentStore.historyNextPage}
      uiLang={uiLang}
    />
  {:else if view === 'bests'}
    <BestsView bests={contentStore.bests} uiLang={uiLang} />
  {:else if view === 'custom'}
    <CustomTextsView
      customTexts={contentStore.customTexts}
      searchText={contentStore.searchText}
      showEditor={contentStore.showEditor}
      newName={contentStore.newName}
      newTextContent={contentStore.newTextContent}
      newLanguage={contentStore.customTextLanguage}
      onSave={() => void contentStore.saveCustomText()}
      onDelete={(id) => void contentStore.deleteCustomText(id)}
      onStart={(id) => void testSession.startCustomTest(id)}
      onSearch={(q) => void contentStore.searchCustom(q)}
      onOpenEditor={(ct) => contentStore.openEditor(ct, testSession.selectedLanguage)}
      onCloseEditor={() => { contentStore.showEditor = false; }}
      onNameChange={(name) => { contentStore.newName = name; }}
      onTextChange={(content) => { contentStore.newTextContent = content; }}
      onLanguageChange={(language) => { contentStore.customTextLanguage = language; }}
      uiLang={uiLang}
    />
  {:else if view === 'settings'}
    <SettingsView
      {settings}
      {themes}
      {activeTheme}
      {uiLang}
      onSelectTheme={settingsStore.selectTheme}
      onUpdateSetting={settingsStore.updateSetting}
    />
  {:else if view === 'lessons'}
    <div class="lesson-lang-selector">
      {#each [['en','EN'],['ru','RU'],['de','DE'],['uk','UK'],['cs','CS'],['pl','PL'],['ro','RO'],['it','IT'],['fr','FR'],['es','ES'],['pt','PT'],['ja','JA'],['zh-hk','繁HK'],['zh-tw','繁TW'],['ko','KO']] as [code, label]}
        <button class:active={contentStore.lessonLang === code} onclick={() => { contentStore.lessonLang = code as typeof contentStore.lessonLang; contentStore.loadLessons(); }}>{label}</button>
      {/each}
    </div>
    <LessonListView
      modules={contentStore.courseModules}
      progress={contentStore.lessonProgress}
      language={contentStore.lessonLang}
      onSelectLesson={(lessonId, language) => void testSession.startLesson(lessonId, language)}
      uiLang={uiLang}
    />
  {:else if view === 'weakkeys'}
    <WeakKeysPanel
      weakKeys={contentStore.weakKeysData}
      charStats={contentStore.weakKeysCharStats}
      onGenerateTraining={() => void testSession.startTraining()}
      {uiLang}
      trainingText={testSession.text}
      trainingCharStatuses={testSession.charStatuses}
      trainingCaretPos={testSession.caretPos}
      trainingRunning={testSession.isRunning}
      trainingLanguage={testSession.isRunning ? testSession.sessionLanguage : testSession.selectedLanguage}
    />
  {:else if view === 'analytics'}
    <AnalyticsView uiLang={uiLang} />
  {:else if view === 'achievements'}
    <AchievementGallery uiLang={uiLang} />
  {/if}
</main>

{#if showOnboarding}
  <OnboardingView
    uiLang={uiLang}
    onUiLangChange={handleOnboardingUiLang}
    onComplete={(result) => void handleOnboardingComplete(result)}
    onSkip={() => void handleOnboardingSkip()}
  />
{/if}

<NotificationStack notifications={notificationStore.notifications} />

{#if cheatsheetOpen}
  <CheatsheetOverlay {uiLang} onClose={() => { cheatsheetOpen = false; }} />
{/if}

{#if vimSearchOpen}
  <VimSearchOverlay
    {uiLang}
    matchCount={searchMatchCount}
    onQuery={applyVimSearch}
    onClose={closeVimSearch}
  />
{/if}

<style>
  :root {
    --color-app-background: #0d0f12;
    --color-surface-primary: #15181d;
    --color-accent: #c5cbd4;
    --color-text-secondary: #adb3bd;
    --color-text-primary: #e7e9ed;
    --color-error: #dc8d8d;
    --color-caret: #f1f3f6;
    --bg: var(--color-app-background);
    --bg-sub: var(--color-surface-primary);
    --main: var(--color-accent);
    --sub: var(--color-text-secondary);
    --text: var(--color-text-primary);
    --error: var(--color-error);
    --caret: var(--color-caret);
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  main {
    display: flex; flex-direction: column; align-items: center;
    min-height: 100vh; gap: 1.5rem; padding: 1rem;
    background-color: var(--bg); color: var(--text);
    font-family: 'JetBrains Mono', monospace;
  }
  .error { color: var(--error); font-size: 0.875rem; }
  .vim-indicator {
    position: fixed; bottom: 0.75rem; right: 0.75rem; z-index: 50;
    padding: 0.2rem 0.6rem; font-size: 0.7rem; font-weight: 700; letter-spacing: 0.08em;
    color: var(--bg); background: var(--main); border-radius: 4px; opacity: 0.85;
  }
  .lesson-lang-selector { display: flex; gap: 0.25rem; }
  .lesson-lang-selector button { background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub); padding: 0.25rem 0.75rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px; }
  .lesson-lang-selector button.active { color: var(--main); border-color: var(--main); }
</style>
