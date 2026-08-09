<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from './lib/api/ipc';
  import { t } from './lib/i18n';
  import { lessonResultNavigation } from './lib/lessonNavigation';
  import { createNavigationStore } from './lib/stores/navigation.svelte';
  import { createNotificationStore } from './lib/stores/notifications.svelte';
  import type {
    CharStatus, EngineOutput, TestSessionResponse, FinalStats, TestSummary,
    PersonalBest, CustomText, AppSettings,
    ThemeInfo, ViewName, ModeName, LanguageCode, ModuleResponse, SessionState,
    DashboardStatsResponse,
  } from './lib/types/index';

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

  // Navigation
  const navigation = createNavigationStore('test');
  const view = $derived(navigation.view);

  // Test state
  let text = $state('');
  let caretPos = $state(0);
  let charStatuses = $state<CharStatus[]>([]);
  let isRunning = $state(false);
  let isComplete = $state(false);
  let sessionState = $state<SessionState>('idle');
  let errorMsg = $state('');
  let liveWpm = $state(0);
  let liveAccuracy = $state(100);
  let elapsedMs = $state(0);
  let finalStats = $state<FinalStats | null>(null);

  // Test config
  let selectedMode = $state<ModeName>('time');
  let selectedDuration = $state(30);
  let selectedWordCount = $state(25);
  let selectedLanguage = $state<LanguageCode>('en');
  let sessionModeType = $state('time');
  let sessionLanguage = $state('en');
  // Backend-issued identity used only as a stale-request correlation token.
  // The backend creates and validates it; the frontend cannot replace it.
  let sessionId = $state<string | null>(null);
  let sessionDurationMs = $state(0);
  let testStartedAt = $state<number | null>(null);

  // History
  let history = $state<TestSummary[]>([]);
  let historyTotal = $state(0);

  // Bests
  let bests = $state<PersonalBest[]>([]);

  // Custom texts
  let customTexts = $state<CustomText[]>([]);
  let editingText = $state<CustomText | null>(null);
  let newName = $state('');
  let newTextContent = $state('');
  let customTextLanguage = $state<LanguageCode>('en');
  let showEditor = $state(false);
  let searchText = $state('');

  // Settings
  let settings = $state<AppSettings | null>(null);
  let uiLang = $state('en');
  let mainFontSize = $derived(`${settings?.font_size ?? 24}px`);

  // Themes
  let themes = $state<ThemeInfo[]>([]);
  let activeTheme = $state('racoon_dark');

  // Lessons
  let courseModules = $state<ModuleResponse[]>([]);
  let lessonProgress = $state<Record<string, { status: string; best_wpm: number; best_accuracy: number }>>({});
  let lessonLang = $state<'en' | 'ru' | 'de' | 'uk' | 'cs' | 'pl' | 'ro' | 'it' | 'fr' | 'es' | 'pt' | 'ja' | 'zh-hk' | 'zh-tw' | 'ko'>('en');
  let currentLessonId = $state<string | null>(null);
  const lessonNavigation = $derived(lessonResultNavigation(courseModules, currentLessonId));

  // Weak Keys
  let weakKeysData = $state<Array<{ ch: string; error_count: number; accuracy: number; rank: number }>>([]);
  let weakKeysCharStats = $state<Record<string, { correct: number; incorrect: number; total: number }>>({});

  // Dashboard
  let dashboardStats = $state<DashboardStatsResponse | null>(null);

  // Zen mode — hide everything except text
  let zenActive = $state(false);

  // Achievement tracking — snapshot before test for auto-toast
  let preTestAchievements = $state<Array<{ id: string; unlocked: boolean }>>([]);

  // Typing warnings
  let lastTypedChar = $state('');
  let capsLockOn = $state(false);

  // Notifications
  const notificationStore = createNotificationStore();

  interface QueuedKey {
    key: string;
    code: string;
    sessionId: string;
    generation: number;
    synthetic: boolean;
  }

  let queuedKeys: QueuedKey[] = [];
  let processingKeys = false;
  let sessionGeneration = 0;
  let timeCompletionQueued = false;
  let audioContext: AudioContext | null = null;

  function applySessionState(nextState: SessionState) {
    sessionState = nextState;
    isRunning = nextState === 'running' || nextState === 'awaiting_persistence' || nextState === 'persisting';
    isComplete = nextState === 'persisted';
  }


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

  function startTestFromResponse(resp: TestSessionResponse, lessonId: string | null = null) {
    sessionGeneration += 1;
    queuedKeys = [];
    timeCompletionQueued = false;
    sessionId = resp.session_id;
    text = resp.text;
    caretPos = 0;
    applySessionState('running');
    finalStats = null;
    sessionModeType = resp.mode_type;
    sessionLanguage = resp.language;
    sessionDurationMs = resp.mode_type === 'time'
      ? Math.max(0, Number(resp.mode_config.duration ?? 0) * 1000)
      : 0;
    currentLessonId = lessonId;
    testStartedAt = null;
    liveWpm = 0;
    liveAccuracy = 100;
    elapsedMs = 0;
    charStatuses = Array.from(resp.text, (ch) => ({
      expected: ch,
      typed: null,
      status: 'pending' as const,
    }));
  }

  function clearAbortedSessionPresentation() {
    sessionGeneration += 1;
    queuedKeys = [];
    timeCompletionQueued = false;
    applySessionState('idle');
    sessionId = null;
    currentLessonId = null;
    testStartedAt = null;
    elapsedMs = 0;
    caretPos = 0;
    charStatuses = [];
  }

  // Replacing a running test is an explicit user action. The backend must
  // accept the abort before any new-session request is sent; this prevents the
  // presentation configuration from diverging from the authoritative engine.
  // A retry-pending completion intentionally cannot be abandoned here.
  async function abandonActiveSessionForReplacement(): Promise<boolean> {
    if (!isRunning || isComplete) return true;
    if (!sessionId) {
      errorMsg = 'Abort error: the active session has no backend identity';
      return false;
    }
    try {
      await ipc.abortSession(sessionId);
      clearAbortedSessionPresentation();
      return true;
    } catch (error) {
      errorMsg = `Abort error: ${ipc.ipcErrorMessage(error)}`;
      return false;
    }
  }

  async function startTest() {
    errorMsg = '';
    finalStats = null;
    if (settings?.zen_mode_enabled) zenActive = true;
    try {
      await snapshotAchievements();
      const params: {
        mode: string;
        language: string;
        duration?: number;
        wordCount?: number;
      } = {
        mode: selectedMode,
        language: selectedLanguage,
      };
      if (selectedMode === 'time') params.duration = selectedDuration;
      if (selectedMode === 'words') params.wordCount = selectedWordCount;

      const resp = await ipc.startTest(params);
      startTestFromResponse(resp);
    } catch (error) {
      zenActive = false;
      errorMsg = `Start test error: ${ipc.ipcErrorMessage(error)}`;
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

  async function finishTest(stats: FinalStats) {
    finalStats = stats;
    zenActive = false;
    testStartedAt = null;
    elapsedMs = stats.duration_ms;

    if (stats.accuracy >= 95) {
      notificationStore.add('SUCCESS', t(uiLang, 'notification.great_result'));
    }

    const lessonId = currentLessonId;
    if (sessionModeType === 'lesson' && lessonId) {
      // The backend persisted lesson completion in the same transaction as the
      // completed test. This local update only renders that confirmed state.
      lessonProgress = {
        ...lessonProgress,
        [lessonId]: {
          status: 'completed',
          best_wpm: Math.max(lessonProgress[lessonId]?.best_wpm ?? 0, stats.wpm),
          best_accuracy: Math.max(lessonProgress[lessonId]?.best_accuracy ?? 0, stats.accuracy),
        },
      };
      void playSound('lesson_complete');
    }
    await checkNewAchievements();
  }

  async function applyEngineOutput(output: EngineOutput, key: string, synthetic: boolean) {
    applySessionState(output.session_state);
    caretPos = output.caret_pos;
    if (output.live_stats) {
      liveWpm = output.live_stats.wpm;
      liveAccuracy = output.live_stats.accuracy;
      elapsedMs = output.live_stats.elapsed_ms;
      testStartedAt = Date.now() - output.live_stats.elapsed_ms;

      if (liveAccuracy >= 95 && output.key_result === 'correct' && Math.random() < 0.05) {
        notificationStore.add('SUCCESS', t(uiLang, 'notification.high_accuracy'));
      }
    }

    if (output.key_result === 'correct' && caretPos > 0) {
      charStatuses[caretPos - 1] = {
        ...charStatuses[caretPos - 1],
        typed: charStatuses[caretPos - 1].expected,
        status: 'correct',
      };
    } else if (output.key_result === 'incorrect' && caretPos < charStatuses.length) {
      charStatuses[caretPos] = { ...charStatuses[caretPos], typed: key, status: 'incorrect' };
    } else if (output.key_result === 'undone_correct' && caretPos < charStatuses.length) {
      charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'backspaced' };
    } else if (output.key_result === 'undone_incorrect' && caretPos < charStatuses.length) {
      charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'pending' };
    }

    if (!synthetic && output.key_result === 'incorrect') {
      void playSound('error');
    } else if (!synthetic && !['noop', 'test_ended'].includes(output.key_result)) {
      void playSound('key_press');
    }

    if (output.test_complete && output.session_state === 'persisted') {
      await finishTest(output.test_complete);
    }
  }

  async function drainKeyQueue() {
    if (processingKeys) return;
    processingKeys = true;
    try {
      while (queuedKeys.length > 0) {
        const queued = queuedKeys.shift();
        if (!queued || queued.generation !== sessionGeneration) continue;
        if (!isRunning || isComplete) continue;
        try {
          const output = await ipc.processKey(queued.key, queued.code, queued.sessionId);
          if (queued.generation !== sessionGeneration) continue;
          await applyEngineOutput(output, queued.key, queued.synthetic);
          if (queued.synthetic && !output.test_complete) timeCompletionQueued = false;
        } catch (error) {
          if (queued.synthetic) timeCompletionQueued = false;
          queuedKeys = [];
          errorMsg = `Typing error: ${ipc.ipcErrorMessage(error)}`;
        }
      }
    } finally {
      processingKeys = false;
      if (queuedKeys.length > 0) void drainKeyQueue();
    }
  }

  function enqueueKey(key: string, code: string, synthetic = false) {
    if (!sessionId) return;
    queuedKeys.push({ key, code, sessionId, generation: sessionGeneration, synthetic });
    void drainKeyQueue();
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

    // Vim mode navigation (only when not actively typing a test)
    if (settings?.vim_mode && !isRunning) {
      const views: ViewName[] = ['dashboard', 'test', 'lessons', 'weakkeys', 'analytics', 'achievements', 'history', 'bests', 'custom', 'settings'];
      const currentIdx = views.indexOf(view);
      if (e.key === 'h' && currentIdx > 0) { e.preventDefault(); switchView(views[currentIdx - 1]); return; }
      if (e.key === 'l' && currentIdx < views.length - 1) { e.preventDefault(); switchView(views[currentIdx + 1]); return; }
      if (e.key === 'k') { e.preventDefault(); window.scrollBy(0, -100); return; }
      if (e.key === 'j') { e.preventDefault(); window.scrollBy(0, 100); return; }
    }

    if (!isRunning || isComplete) return;

    // Caps Lock detection
    if (e.getModifierState && e.getModifierState('CapsLock') !== capsLockOn) {
      capsLockOn = e.getModifierState('CapsLock');
      if (capsLockOn && settings?.show_capslock_warnings) {
        notificationStore.add('WARNING', t(uiLang, 'warning.caps_title'));
      }
    }

    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;
    if (e.key === 'Backspace' || e.key === 'Tab' || e.key === ' ' || e.key.length === 1) e.preventDefault();

    // Track last typed char for layout detection
    if (e.key.length === 1) {
      lastTypedChar = e.key;
      testStartedAt ??= Date.now();
    }
    enqueueKey(e.key, e.code);
  }

  $effect(() => {
    const startedAt = testStartedAt;
    const running = isRunning;
    const complete = isComplete;
    const mode = sessionModeType;
    const durationMs = sessionDurationMs;
    if (!running || complete || startedAt === null) return;

    const updateClock = () => {
      const currentElapsed = Math.max(0, Date.now() - startedAt);
      elapsedMs = mode === 'time' && durationMs > 0
        ? Math.min(currentElapsed, durationMs)
        : currentElapsed;
      if (
        mode === 'time'
        && durationMs > 0
        && currentElapsed >= durationMs
        && !timeCompletionQueued
      ) {
        timeCompletionQueued = true;
        enqueueKey('', '', true);
      }
    };

    updateClock();
    const interval = window.setInterval(updateClock, 50);
    return () => window.clearInterval(interval);
  });

  async function abortTest() {
    if (!isRunning) return;
    await abandonActiveSessionForReplacement();
  }

  async function loadHistory() {
    const r = await ipc.getStatsHistory(20);
    history = r.tests;
    historyTotal = r.total;
  }

  async function loadBests() {
    bests = await ipc.getPersonalBests();
  }

  async function loadCustomTexts() {
    customTexts = await ipc.getCustomTexts(50);
  }

  async function loadSettings() {
    settings = await ipc.getSettings();
    activeTheme = settings.theme;
    uiLang = settings.ui_language || 'en';
    await applyTheme(activeTheme);
  }

  async function loadThemes() {
    themes = await ipc.getThemes();
  }

  async function applyTheme(name: string) {
    const css = await ipc.getThemeCss(name);
    const styleEl = document.getElementById('theme-style') || (() => {
      const el = document.createElement('style');
      el.id = 'theme-style';
      document.head.appendChild(el);
      return el;
    })();
    styleEl.setAttribute('data-theme', name);
    styleEl.textContent = css;

    // Apply variables inline as well as through the stylesheet. This keeps
    // theme switching reliable when component-scoped CSS is present.
    const root = document.documentElement;
    const variables = /--([a-z0-9-]+)\s*:\s*(#[0-9a-fA-F]{3,8})\s*;/g;
    for (const match of css.matchAll(variables)) {
      root.style.setProperty(`--${match[1]}`, match[2], 'important');
    }
    root.dataset.theme = name;
    const themeInfo = themes.find((theme) => theme.name === name);
    root.style.colorScheme = themeInfo?.is_dark ? 'dark' : 'light';
  }

  async function selectTheme(name: string) {
    try {
      await applyTheme(name);
      await ipc.setSetting('theme', name);
      activeTheme = name;
      settings = await ipc.getSettings();
      errorMsg = '';
    } catch (error) {
      const detail = error instanceof Error
        ? error.message
        : typeof error === 'object' && error !== null
          ? JSON.stringify(error)
          : String(error);
      errorMsg = `Theme error: ${detail}`;
      console.error('Theme switch failed', { name, error });
    }
  }

  async function updateSetting(key: string, value: unknown) {
    try {
      await ipc.setSetting(key, value);
      settings = await ipc.getSettings();
    } catch (error) {
      errorMsg = `Settings error: ${ipc.ipcErrorMessage(error)}`;
      return;
    }
    if (key === 'ui_language') {
      uiLang = (value as string) || 'en';
    }
  }

  function openEditor(ct: CustomText | null) {
    editingText = ct;
    newName = ct ? ct.name : '';
    newTextContent = ct ? ct.text : '';
    customTextLanguage = ct?.language ?? selectedLanguage;
    showEditor = true;
  }

  async function saveCustomText() {
    try {
      if (editingText) {
        await ipc.updateCustomText(editingText.id, newName, newTextContent, customTextLanguage);
      } else {
        await ipc.saveCustomText(newName, newTextContent, customTextLanguage);
      }
      showEditor = false;
      await loadCustomTexts();
    } catch (err) {
      errorMsg = `Save error: ${err}`;
    }
  }

  async function deleteCustomText(id: number) {
    await ipc.deleteCustomText(id);
    await loadCustomTexts();
  }

  async function startCustomTest(id: number) {
    try {
      if (!(await abandonActiveSessionForReplacement())) return;
      await snapshotAchievements();
      const resp = await ipc.startCustomTextTest(id);
      startTestFromResponse(resp);
      switchView('test');
    } catch (error) {
      errorMsg = `Start custom text error: ${error}`;
    }
  }

  async function searchCustom(q: string) {
    searchText = q;
    if (q.trim()) {
      customTexts = await ipc.searchCustomTexts(q, 20);
    } else {
      await loadCustomTexts();
    }
  }

  function switchView(v: ViewName) {
    navigation.navigate(v);
    if (v === 'history') loadHistory();
    if (v === 'bests') loadBests();
    if (v === 'custom') loadCustomTexts();
    if (v === 'lessons') loadLessons();
    if (v === 'weakkeys') loadWeakKeys();
    if (v === 'dashboard') loadDashboard();
  }

  async function loadDashboard() {
    try {
      dashboardStats = await ipc.getDashboardStats();
    } catch (e) {
      errorMsg = `Dashboard error: ${e}`;
    }
  }

  async function loadWeakKeys() {
    try {
      const data = await ipc.analyzeWeakKeys();
      weakKeysData = data.weak_keys || [];
      // Populate per-key stats from aggregated heatmap so KeyboardTrainer
      // coloring (weak-critical / weak-warning) activates in WeakKeysPanel.
      try {
        const heatmap = await ipc.getAggregatedHeatmap(50);
        // Convert KeyHeatData → CharStat shape expected by KeyboardTrainer.
        weakKeysCharStats = Object.fromEntries(
          Object.entries(heatmap).map(([k, v]) => [
            k,
            { correct: v.correct, incorrect: v.incorrect, total: v.total_attempts },
          ]),
        );
      } catch {
        // Aggregated heatmap is best-effort; ignore if unavailable.
      }
    } catch (e) {
      errorMsg = `Weak keys error: ${e}`;
    }
  }

  async function onGenerateTraining() {
    try {
      if (!(await abandonActiveSessionForReplacement())) return;
      await snapshotAchievements();
      const generatedText = await ipc.generateWeakKeysTraining(selectedLanguage, 25);
      const resp = await ipc.startTest({
        mode: 'custom',
        language: selectedLanguage,
        text: generatedText,
      });
      startTestFromResponse(resp);
    } catch (e) {
      errorMsg = `Training error: ${e}`;
    }
  }

  async function loadLessons() {
    try {
      const course = await ipc.getCourse(lessonLang);
      courseModules = course.modules;
      const progress = await ipc.getLessonProgress(lessonLang);
      lessonProgress = Object.fromEntries(
        progress.map((lesson) => [lesson.lesson_id, {
          status: lesson.status,
          best_wpm: lesson.best_wpm,
          best_accuracy: lesson.best_accuracy,
        }]),
      );
    } catch (e) {
      errorMsg = `Lessons error: ${e}`;
    }
  }

  async function onSelectLesson(lessonId: string, language: string) {
    try {
      if (!(await abandonActiveSessionForReplacement())) return;
      await snapshotAchievements();
      const resp = await ipc.startLesson(lessonId, language);
      startTestFromResponse(resp, lessonId);
      switchView('test');
    } catch (e) {
      errorMsg = `Start lesson error: ${e}`;
    }
  }

  async function onRepeatLesson() {
    if (!currentLessonId) return;
    await onSelectLesson(currentLessonId, lessonLang);
  }

  async function onNextLesson() {
    const next = lessonNavigation?.nextLessonId;
    if (!next) return;
    await onSelectLesson(next, lessonLang);
  }

  function onReturnToLessons() {
    currentLessonId = null;
    switchView('lessons');
  }

  async function updateTestConfigurationAndRestart(update: () => void) {
    if (!(await abandonActiveSessionForReplacement())) return;
    update();
    await startTest();
  }

  function onModeChange(m: ModeName) {
    void updateTestConfigurationAndRestart(() => { selectedMode = m; });
  }

  function onDurationChange(d: number) {
    void updateTestConfigurationAndRestart(() => { selectedDuration = d; });
  }

  function onWordCountChange(w: number) {
    void updateTestConfigurationAndRestart(() => { selectedWordCount = w; });
  }

  function onLanguageChange(l: LanguageCode) {
    void updateTestConfigurationAndRestart(() => { selectedLanguage = l; });
  }

  onMount(async () => {
    try {
      await loadThemes();
    } catch (error) {
      errorMsg = `Theme catalog error: ${ipc.ipcErrorMessage(error)}`;
    }
    try {
      await loadSettings();
    } catch (error) {
      errorMsg = `Settings load error: ${ipc.ipcErrorMessage(error)}`;
    }
    await startTest();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<main style:font-size={mainFontSize} data-session-state={sessionState}>
  {#if !zenActive}
    <NavigationBar {view} {historyTotal} {uiLang} onNavigate={switchView} />
  {/if}

  {#if errorMsg}
    <p class="error">{errorMsg}</p>
  {/if}

  {#if view === 'dashboard'}
    <DashboardView
      stats={dashboardStats}
      onNavigate={(v) => switchView(v as ViewName)}
      weakKeys={weakKeysData}
      onStartTraining={onGenerateTraining}
      uiLang={uiLang}
    />
  {:else if view === 'test'}
    {#if isRunning}
      <TypingWarnings
        expectedLanguage={sessionLanguage}
        {lastTypedChar}
        {capsLockOn}
        showLayoutWarnings={true}
        showCapsLockWarnings={settings?.show_capslock_warnings ?? true}
        {uiLang}
      />
    {/if}
    <TestView
      {text}
      {caretPos}
      {charStatuses}
      {isRunning}
      {isComplete}
      {liveWpm}
      {liveAccuracy}
      {elapsedMs}
      {finalStats}
      {settings}
      {selectedMode}
      {selectedDuration}
      {selectedWordCount}
      {selectedLanguage}
      {sessionModeType}
      {sessionLanguage}
      onModeChange={onModeChange}
      onDurationChange={onDurationChange}
      onWordCountChange={onWordCountChange}
      onLanguageChange={onLanguageChange}
      onAbort={abortTest}
      onRestart={startTest}
      {lessonNavigation}
      {onRepeatLesson}
      {onNextLesson}
      {onReturnToLessons}
      uiLang={uiLang}
    />
  {:else if view === 'history'}
    <HistoryView {history} total={historyTotal} uiLang={uiLang} />
  {:else if view === 'bests'}
    <BestsView {bests} uiLang={uiLang} />
  {:else if view === 'custom'}
    <CustomTextsView
      {customTexts}
      {searchText}
      {showEditor}
      {newName}
      {newTextContent}
      newLanguage={customTextLanguage}
      onSave={saveCustomText}
      onDelete={deleteCustomText}
      onStart={startCustomTest}
      onSearch={searchCustom}
      onOpenEditor={openEditor}
      onCloseEditor={() => { showEditor = false; }}
      onNameChange={(name) => { newName = name; }}
      onTextChange={(content) => { newTextContent = content; }}
      onLanguageChange={(language) => { customTextLanguage = language; }}
      uiLang={uiLang}
    />
  {:else if view === 'settings'}
    <SettingsView
      {settings}
      {themes}
      {activeTheme}
      {uiLang}
      onSelectTheme={selectTheme}
      onUpdateSetting={updateSetting}
    />
  {:else if view === 'lessons'}
    <div class="lesson-lang-selector">
      {#each [['en','EN'],['ru','RU'],['de','DE'],['uk','UK'],['cs','CS'],['pl','PL'],['ro','RO'],['it','IT'],['fr','FR'],['es','ES'],['pt','PT'],['ja','JA'],['zh-hk','繁HK'],['zh-tw','繁TW'],['ko','KO']] as [code, label]}
        <button class:active={lessonLang === code} onclick={() => { lessonLang = code as typeof lessonLang; loadLessons(); }}>{label}</button>
      {/each}
    </div>
    <LessonListView
      modules={courseModules}
      progress={lessonProgress}
      language={lessonLang}
      onSelectLesson={onSelectLesson}
      uiLang={uiLang}
    />
  {:else if view === 'weakkeys'}
    <WeakKeysPanel
      weakKeys={weakKeysData}
      charStats={weakKeysCharStats}
      onGenerateTraining={onGenerateTraining}
      {uiLang}
      trainingText={text}
      trainingCharStatuses={charStatuses}
      trainingCaretPos={caretPos}
      trainingRunning={isRunning}
      trainingLanguage={isRunning ? sessionLanguage : selectedLanguage}
    />
  {:else if view === 'analytics'}
    <AnalyticsView uiLang={uiLang} />
  {:else if view === 'achievements'}
    <AchievementGallery uiLang={uiLang} />
  {/if}
</main>

<NotificationStack notifications={notificationStore.notifications} />

<style>
  :root {
    --bg: #151a24; --bg-sub: #202a38; --main: #5eead4;
    --sub: #7890a8; --text: #e8f0f7; --error: #fb7185; --caret: #fbbf24;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  main {
    display: flex; flex-direction: column; align-items: center;
    min-height: 100vh; gap: 1.5rem; padding: 1rem;
    background-color: var(--bg); color: var(--text);
    font-family: 'JetBrains Mono', monospace;
  }
  .error { color: var(--error); font-size: 0.875rem; }
  .lesson-lang-selector { display: flex; gap: 0.25rem; }
  .lesson-lang-selector button { background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub); padding: 0.25rem 0.75rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px; }
  .lesson-lang-selector button.active { color: var(--main); border-color: var(--main); }
</style>
