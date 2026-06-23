<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import * as ipc from './lib/api/ipc';
  import type {
    CharStatus, TestSessionResponse, FinalStats, TestSummary,
    StatsHistoryResponse, PersonalBest, CustomText, AppSettings,
    ThemeInfo, ViewName, ModeName, LanguageCode, ModuleResponse,
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

  // Navigation
  let view = $state<ViewName>('test');

  // Test state
  let text = $state('');
  let caretPos = $state(0);
  let charStatuses = $state<CharStatus[]>([]);
  let isRunning = $state(false);
  let isComplete = $state(false);
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
  let showEditor = $state(false);
  let searchText = $state('');

  // Settings
  let settings = $state<AppSettings | null>(null);

  // Themes
  let themes = $state<ThemeInfo[]>([]);
  let activeTheme = $state('serika_dark');

  // Lessons
  let courseModules = $state<ModuleResponse[]>([]);
  let lessonProgress = $state<Record<string, { status: string; best_wpm: number; best_accuracy: number }>>({});
  let lessonLang = $state<'en' | 'ru'>('en');

  // Weak Keys
  let weakKeysData = $state<Array<{ ch: string; error_count: number; accuracy: number; rank: number }>>([]);
  let weakKeysCharStats = $state<Record<string, { correct: number; incorrect: number; total: number }>>({});

  // Dashboard
  let dashboardStats = $state<DashboardStatsResponse | null>(null);

  // Zen mode — hide everything except text
  let zenActive = $state(false);

  // Typing warnings
  let lastTypedChar = $state('');
  let capsLockOn = $state(false);

  // Notifications
  let notifications = $state<Array<{ id: number; type: string; message: string; timestamp: number }>>([]);
  let notifId = 0;

  function addNotification(type: string, message: string) {
    const id = ++notifId;
    notifications = [...notifications, { id, type, message, timestamp: Date.now() }];
    setTimeout(() => {
      notifications = notifications.filter(n => n.id !== id);
    }, 5000);
  }

  async function startTest() {
    errorMsg = '';
    finalStats = null;
    if (settings?.zen_mode_enabled) zenActive = true;
    const params: Record<string, unknown> = {
      mode: selectedMode,
      language: selectedLanguage,
    };
    if (selectedMode === 'time') params.duration = selectedDuration;
    if (selectedMode === 'words') params.wordCount = selectedWordCount;

    const resp = await ipc.startTest(params as any);
    text = resp.text;
    caretPos = 0;
    isComplete = false;
    isRunning = true;
    sessionModeType = resp.mode_type;
    sessionLanguage = resp.language;
    liveWpm = 0;
    liveAccuracy = 100;
    elapsedMs = 0;
    charStatuses = text.split('').map((ch) => ({ expected: ch, typed: null, status: 'pending' as const }));
  }

  async function handleKeydown(e: KeyboardEvent) {
    if (!isRunning || isComplete) return;

    // Caps Lock detection
    if (e.getModifierState && e.getModifierState('CapsLock') !== capsLockOn) {
      capsLockOn = e.getModifierState('CapsLock');
      if (capsLockOn && settings?.show_capslock_warnings) {
        addNotification('WARNING', 'Caps Lock включён');
      }
    }

    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;
    if (e.key === 'Backspace' || e.key === 'Tab' || e.key === ' ' || e.key.length === 1) e.preventDefault();

    // Track last typed char for layout detection
    if (e.key.length === 1) {
      lastTypedChar = e.key;
    }

    try {
      const output = await ipc.processKey(e.key, e.code);
      caretPos = output.caret_pos;
      if (output.live_stats) {
        liveWpm = output.live_stats.wpm;
        liveAccuracy = output.live_stats.accuracy;
        elapsedMs = output.live_stats.elapsed_ms;

        // Smart notifications
        if (liveAccuracy >= 95 && output.key_result === 'correct' && Math.random() < 0.05) {
          addNotification('SUCCESS', 'Точность выше 95%');
        }
      }
      if (output.key_result === 'correct' && caretPos > 0) {
        charStatuses[caretPos - 1] = { ...charStatuses[caretPos - 1], typed: charStatuses[caretPos - 1].expected, status: 'correct' };
      } else if (output.key_result === 'incorrect' && caretPos < charStatuses.length) {
        charStatuses[caretPos] = { ...charStatuses[caretPos], typed: e.key, status: 'incorrect' };
      } else if (output.key_result === 'undone_correct' && caretPos < charStatuses.length) {
        charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'backspaced' };
      } else if (output.key_result === 'undone_incorrect' && caretPos < charStatuses.length) {
        charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'pending' };
      }
      if (output.test_complete) {
        finalStats = output.test_complete;
        isComplete = true;
        zenActive = false;
        isRunning = false;
        if (finalStats.accuracy >= 95) {
          addNotification('SUCCESS', 'Отличный результат!');
        }
      }
    } catch (err) {
      errorMsg = `Error: ${err}`;
    }
  }

  function abortTest() {
    if (isRunning) ipc.abortSession().catch(() => {});
    isRunning = false;
    isComplete = false;
    caretPos = 0;
    charStatuses = [];
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
    styleEl.textContent = css;
  }

  async function selectTheme(name: string) {
    activeTheme = name;
    await ipc.setSetting('theme', name);
    await applyTheme(name);
    settings = await ipc.getSettings();
  }

  async function updateSetting(key: string, value: unknown) {
    await ipc.setSetting(key, value);
    settings = await ipc.getSettings();
  }

  function openEditor(ct: CustomText | null) {
    editingText = ct;
    newName = ct ? ct.name : '';
    newTextContent = ct ? ct.text : '';
    showEditor = true;
  }

  async function saveCustomText() {
    try {
      if (editingText) {
        await ipc.updateCustomText(editingText.id, newName, newTextContent);
      } else {
        await ipc.saveCustomText(newName, newTextContent);
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
    const resp = await ipc.startCustomTextTest(id);
    text = resp.text;
    caretPos = 0;
    isComplete = false;
    isRunning = true;
    finalStats = null;
    sessionModeType = resp.mode_type;
    sessionLanguage = resp.language;
    charStatuses = text.split('').map((ch) => ({ expected: ch, typed: null, status: 'pending' as const }));
    view = 'test';
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
    view = v;
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
      const data = await ipc.analyzeWeakKeys() as { weak_keys: Array<{ ch: string; error_count: number; accuracy: number; rank: number }> };
      weakKeysData = data.weak_keys || [];
    } catch (e) {
      errorMsg = `Weak keys error: ${e}`;
    }
  }

  async function onGenerateTraining() {
    try {
      const text = await ipc.generateWeakKeysTraining(selectedLanguage, 25);
      // Start a test with this text
      const resp = await ipc.startTest({ mode: 'custom', language: selectedLanguage, text });
      // ... use resp to start test
      view = 'test';
    } catch (e) {
      errorMsg = `Training error: ${e}`;
    }
  }

  async function loadLessons() {
    try {
      const course = await ipc.getCourse(lessonLang);
      courseModules = course.modules;
      const p = await ipc.getLessonProgress(lessonLang) as { modules: Array<{ module_id: string; completed_lessons: number }> };
      // Progress is course-level, we need per-lesson. For now, use empty.
      lessonProgress = {};
    } catch (e) {
      errorMsg = `Lessons error: ${e}`;
    }
  }

  async function onSelectLesson(lessonId: string, language: string) {
    try {
      const resp = await ipc.startLesson(lessonId, language);
      text = resp.text;
      caretPos = 0;
      isComplete = false;
      isRunning = true;
      finalStats = null;
      sessionModeType = resp.mode_type;
      sessionLanguage = resp.language;
      charStatuses = text.split('').map((ch) => ({ expected: ch, typed: null, status: 'pending' as const }));
      view = 'test';
    } catch (e) {
      errorMsg = `Start lesson error: ${e}`;
    }
  }

  function onModeChange(m: ModeName) {
    selectedMode = m;
    startTest();
  }

  function onDurationChange(d: number) {
    selectedDuration = d;
    startTest();
  }

  function onWordCountChange(w: number) {
    selectedWordCount = w;
    startTest();
  }

  function onLanguageChange(l: LanguageCode) {
    selectedLanguage = l;
    startTest();
  }

  onMount(async () => {
    await loadThemes();
    await loadSettings();
    await startTest();
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<main>
  {#if !zenActive}
    <NavigationBar {view} {historyTotal} onNavigate={switchView} />
  {/if}

  {#if errorMsg}
    <p class="error">{errorMsg}</p>
  {/if}

  {#if view === 'dashboard'}
    <DashboardView stats={dashboardStats} onNavigate={(v) => switchView(v as ViewName)} />
  {:else if view === 'test'}
    {#if isRunning && settings?.show_layout_warnings}
      <TypingWarnings
        expectedLanguage={sessionLanguage}
        {lastTypedChar}
        {capsLockOn}
        showLayoutWarnings={settings.show_layout_warnings}
        showCapsLockWarnings={settings.show_capslock_warnings}
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
    />
  {:else if view === 'history'}
    <HistoryView {history} total={historyTotal} />
  {:else if view === 'bests'}
    <BestsView {bests} />
  {:else if view === 'custom'}
    <CustomTextsView
      {customTexts}
      {searchText}
      {showEditor}
      {newName}
      {newTextContent}
      onSave={saveCustomText}
      onDelete={deleteCustomText}
      onStart={startCustomTest}
      onSearch={searchCustom}
      onOpenEditor={openEditor}
      onCloseEditor={() => { showEditor = false; }}
    />
  {:else if view === 'settings'}
    <SettingsView
      {settings}
      {themes}
      {activeTheme}
      onSelectTheme={selectTheme}
      onUpdateSetting={updateSetting}
    />
  {:else if view === 'lessons'}
    <div class="lesson-lang-selector">
      <button class:active={lessonLang === 'en'} onclick={() => { lessonLang = 'en'; loadLessons(); }}>EN</button>
      <button class:active={lessonLang === 'ru'} onclick={() => { lessonLang = 'ru'; loadLessons(); }}>RU</button>
    </div>
    <LessonListView
      modules={courseModules}
      progress={lessonProgress}
      language={lessonLang}
      onSelectLesson={onSelectLesson}
    />
  {:else if view === 'weakkeys'}
    <WeakKeysPanel
      weakKeys={weakKeysData}
      charStats={weakKeysCharStats}
      onGenerateTraining={onGenerateTraining}
    />
  {:else if view === 'analytics'}
    <AnalyticsView />
  {/if}
</main>

<NotificationStack {notifications} />

<style>
  :root {
    --bg: #323437; --bg-sub: #2c2e31; --main: #e2b714;
    --sub: #555555; --text: #999999; --error: #ca4754; --caret: #e2b714;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  main {
    display: flex; flex-direction: column; align-items: center;
    min-height: 100vh; gap: 1.5rem; padding: 1rem;
    background-color: var(--bg); color: var(--text);
    font-family: 'JetBrains Mono', monospace; font-size: 24px;
  }
  .error { color: var(--error); font-size: 0.875rem; }
  .lesson-lang-selector { display: flex; gap: 0.25rem; }
  .lesson-lang-selector button { background: var(--bg-sub); color: var(--sub); border: 1px solid var(--sub); padding: 0.25rem 0.75rem; font-family: inherit; font-size: 0.75rem; cursor: pointer; border-radius: 4px; }
  .lesson-lang-selector button.active { color: var(--main); border-color: var(--main); }
</style>