// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Feature store for the typing-session lifecycle: presentation state, test
// configuration, the key queue, and the IPC orchestration around start,
// input, abort, and completion. Side effects (sound, toasts, lesson/history/
// achievement updates, view switching) are injected by the App component so
// the store stays transport-adjacent but presentation-owned.

import * as ipc from '../api/ipc';
import { t } from '../i18n';
import type {
  AppSettings,
  CharStatus,
  EngineOutput,
  FinalStats,
  LanguageCode,
  ModeName,
  SessionState,
  TestSessionResponse,
} from '../types/index';

export type SoundEventName = 'key_press' | 'error' | 'lesson_complete';

export interface TestSessionDeps {
  getSettings: () => AppSettings | null;
  getUiLang: () => string;
  setError: (message: string) => void;
  setZenActive: (active: boolean) => void;
  playSound: (event: SoundEventName) => void;
  notify: (type: 'SUCCESS' | 'WARNING', message: string) => void;
  beforeStart: () => Promise<void>;
  getCurrentLessonId: () => string | null;
  setCurrentLessonId: (lessonId: string | null) => void;
  onLessonCompleted: (lessonId: string, stats: FinalStats) => void;
  onHistoryChanged: () => void;
  onAchievementsChanged: () => Promise<void>;
  onStarted: () => void;
}

interface QueuedKey {
  key: string;
  code: string;
  sessionId: string;
  generation: number;
  synthetic: boolean;
}

export function createTestSessionStore(deps: TestSessionDeps) {
  // Test state
  let text = $state('');
  let caretPos = $state(0);
  let charStatuses = $state<CharStatus[]>([]);
  let isRunning = $state(false);
  let isComplete = $state(false);
  // Guards against a race where two startTest() calls both observe isRunning
  // === false while the first is still awaiting the backend, causing a second
  // IPC start that the backend rejects with TEST_ALREADY_ACTIVE.
  let startingTest = $state(false);
  let sessionState = $state<SessionState>('idle');
  let liveWpm = $state(0);
  let liveAccuracy = $state(100);
  let elapsedMs = $state(0);
  let finalStats = $state<FinalStats | null>(null);
  // Позиции, где была допущена ошибка — хвост ошибки остаётся видимым
  // после backspace/ретайпа до конца теста.
  let erroredPositions = $state(new Set<number>());

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

  // Typing warnings
  let lastTypedChar = $state('');
  let capsLockOn = $state(false);

  // Key queue (not rendered — plain state)
  let queuedKeys: QueuedKey[] = [];
  let processingKeys = false;
  let sessionGeneration = 0;
  let timeCompletionQueued = false;
  // Timestamp of the last "accuracy above 95%" toast — debounce (30s).
  let lastHighAccToastAt = 0;

  function applySessionState(nextState: SessionState) {
    sessionState = nextState;
    isRunning = nextState === 'running' || nextState === 'awaiting_persistence' || nextState === 'persisting';
    isComplete = nextState === 'persisted';
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
    deps.setCurrentLessonId(lessonId);
    testStartedAt = null;
    liveWpm = 0;
    liveAccuracy = 100;
    elapsedMs = 0;
    // Сброс layout/caps-детекции: иначе после RU-теста новый EN-тест
    // показывает карточку «неверная раскладка» до первого нажатия.
    lastTypedChar = '';
    capsLockOn = false;
    erroredPositions = new Set();
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
    deps.setCurrentLessonId(null);
    testStartedAt = null;
    elapsedMs = 0;
    caretPos = 0;
    charStatuses = [];
    erroredPositions = new Set();
  }

  // Replacing a running test is an explicit user action. The backend must
  // accept the abort before any new-session request is sent; this prevents the
  // presentation configuration from diverging from the authoritative engine.
  // A retry-pending completion intentionally cannot be abandoned here.
  async function abandonActiveSessionForReplacement(): Promise<boolean> {
    if (!isRunning || isComplete) return true;
    if (!sessionId) {
      deps.setError('Abort error: the active session has no backend identity');
      return false;
    }
    try {
      await ipc.abortSession(sessionId);
      clearAbortedSessionPresentation();
      return true;
    } catch (error) {
      deps.setError(`Abort error: ${ipc.ipcErrorMessage(error)}`);
      return false;
    }
  }

  async function startTest() {
    // Guard against a double start (e.g. a racing onMount + user click). The
    // backend rejects a second start with TEST_ALREADY_ACTIVE; surface a
    // clear message instead of a raw IPC error. `startingTest` closes the gap
    // where isRunning is still false while the first start is in flight.
    if (isRunning && !isComplete) {
      deps.setError('Start test error: A test is already running.');
      return;
    }
    if (startingTest) return;
    startingTest = true;
    deps.setError('');
    finalStats = null;
    if (deps.getSettings()?.zen_mode_enabled) deps.setZenActive(true);
    try {
      await deps.beforeStart();
      const params: {
        mode: ModeName;
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
      deps.setZenActive(false);
      deps.setError(`Start test error: ${ipc.ipcErrorMessage(error)}`);
    } finally {
      startingTest = false;
    }
  }

  async function finishTest(stats: FinalStats) {
    finalStats = stats;
    deps.setZenActive(false);
    testStartedAt = null;
    elapsedMs = stats.duration_ms;

    if (stats.accuracy >= 95) {
      deps.notify('SUCCESS', t(deps.getUiLang(), 'notification.great_result'));
    }

    if (sessionModeType === 'lesson') {
      const lessonId = deps.getCurrentLessonId();
      if (lessonId) {
        deps.onLessonCompleted(lessonId, stats);
        deps.playSound('lesson_complete');
      }
    }
    // Держим счётчик истории актуальным для бейджа в навигации.
    deps.onHistoryChanged();
    await deps.onAchievementsChanged();
  }

  async function applyEngineOutput(output: EngineOutput, key: string, synthetic: boolean) {
    applySessionState(output.session_state);
    caretPos = output.caret_pos;
    if (output.live_stats) {
      liveWpm = output.live_stats.wpm;
      liveAccuracy = output.live_stats.accuracy;
      elapsedMs = output.live_stats.elapsed_ms;
      testStartedAt = Date.now() - output.live_stats.elapsed_ms;

      // Toast не чаще раза в 30 секунд — иначе спам на каждый 20-й keystroke.
      const now = Date.now();
      if (
        liveAccuracy >= 95 && output.key_result === 'correct' && Math.random() < 0.05
        && now - lastHighAccToastAt > 30_000
      ) {
        lastHighAccToastAt = now;
        deps.notify('SUCCESS', t(deps.getUiLang(), 'notification.high_accuracy'));
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
      erroredPositions.add(caretPos);
    } else if (output.key_result === 'undone_correct' && caretPos < charStatuses.length) {
      charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'backspaced' };
    } else if (output.key_result === 'undone_incorrect' && caretPos < charStatuses.length) {
      charStatuses[caretPos] = { ...charStatuses[caretPos], typed: null, status: 'pending' };
    }

    if (!synthetic && output.key_result === 'incorrect') {
      deps.playSound('error');
    } else if (!synthetic && !['noop', 'test_ended'].includes(output.key_result)) {
      deps.playSound('key_press');
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
          deps.setError(`Typing error: ${ipc.ipcErrorMessage(error)}`);
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

  async function abortTest() {
    if (!isRunning) return;
    await abandonActiveSessionForReplacement();
  }

  // Restart the current test: abort the active session (if any) then start a
  // fresh one. Used by the in-test "Restart" button.
  async function restartTest() {
    if (!(await abandonActiveSessionForReplacement())) return;
    await startTest();
  }

  async function startCustomTest(id: number) {
    if (startingTest) return;
    startingTest = true;
    try {
      if (!(await abandonActiveSessionForReplacement())) return;
      await deps.beforeStart();
      const resp = await ipc.startCustomTextTest(id);
      startTestFromResponse(resp);
      deps.onStarted();
    } catch (error) {
      deps.setError(`Start custom text error: ${error}`);
    } finally {
      startingTest = false;
    }
  }

  async function startLesson(lessonId: string, language: string) {
    if (startingTest) return;
    startingTest = true;
    try {
      if (!(await abandonActiveSessionForReplacement())) return;
      await deps.beforeStart();
      const resp = await ipc.startLesson(lessonId, language);
      startTestFromResponse(resp, lessonId);
      deps.onStarted();
    } catch (e) {
      deps.setError(`Start lesson error: ${e}`);
    } finally {
      startingTest = false;
    }
  }

  async function startTraining() {
    if (startingTest) return;
    startingTest = true;
    try {
      if (!(await abandonActiveSessionForReplacement())) return;
      await deps.beforeStart();
      const generatedText = await ipc.generateWeakKeysTraining(selectedLanguage, 25);
      const resp = await ipc.startTest({
        mode: 'custom',
        language: selectedLanguage,
        text: generatedText,
      });
      startTestFromResponse(resp);
    } catch (e) {
      deps.setError(`Training error: ${e}`);
    } finally {
      startingTest = false;
    }
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

  return {
    get text() { return text; },
    get caretPos() { return caretPos; },
    get charStatuses() { return charStatuses; },
    get isRunning() { return isRunning; },
    get isComplete() { return isComplete; },
    get sessionState() { return sessionState; },
    get liveWpm() { return liveWpm; },
    get liveAccuracy() { return liveAccuracy; },
    get elapsedMs() { return elapsedMs; },
    get finalStats() { return finalStats; },
    get erroredPositions() { return erroredPositions; },
    get selectedMode() { return selectedMode; },
    get selectedDuration() { return selectedDuration; },
    get selectedWordCount() { return selectedWordCount; },
    get selectedLanguage() { return selectedLanguage; },
    get sessionModeType() { return sessionModeType; },
    get sessionLanguage() { return sessionLanguage; },
    get sessionDurationMs() { return sessionDurationMs; },
    get testStartedAt() { return testStartedAt; },
    get lastTypedChar() { return lastTypedChar; },
    get capsLockOn() { return capsLockOn; },
    get timeCompletionQueued() { return timeCompletionQueued; },
    set timeCompletionQueued(value: boolean) { timeCompletionQueued = value; },
    set lastTypedChar(value: string) { lastTypedChar = value; },
    set capsLockOn(value: boolean) { capsLockOn = value; },
    set testStartedAt(value: number | null) { testStartedAt = value; },
    set elapsedMs(value: number) { elapsedMs = value; },
    startTest,
    startTestFromResponse,
    clearAbortedSessionPresentation,
    abandonActiveSessionForReplacement,
    abortTest,
    restartTest,
    startCustomTest,
    startLesson,
    startTraining,
    enqueueKey,
    onModeChange,
    onDurationChange,
    onWordCountChange,
    onLanguageChange,
  };
}
