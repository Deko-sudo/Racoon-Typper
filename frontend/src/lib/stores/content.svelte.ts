// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Feature store for read-mostly content views: history, personal bests,
// custom texts, lessons, weak keys, and the dashboard. Each loader is
// independent; the App component triggers loads on view switches.

import * as ipc from '../api/ipc';
import { lessonResultNavigation } from '../lessonNavigation';
import type {
  CustomText,
  DashboardStatsResponse,
  LanguageCode,
  ModuleResponse,
  PersonalBest,
  TestSummary,
  WeeklySummaryResponse,
} from '../types/index';

export type LessonLanguage = 'en' | 'ru' | 'de' | 'uk' | 'cs' | 'pl' | 'ro' | 'it' | 'fr' | 'es' | 'pt' | 'ja' | 'zh-hk' | 'zh-tw' | 'ko';

export interface ContentStoreDeps {
  setError: (message: string) => void;
}

export function createContentStore(deps: ContentStoreDeps) {
  // History
  let history = $state<TestSummary[]>([]);
  let historyTotal = $state(0);
  let historyPage = $state(0);

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

  // Lessons
  let courseModules = $state<ModuleResponse[]>([]);
  let lessonProgress = $state<Record<string, { status: string; best_wpm: number; best_accuracy: number }>>({});
  let lessonLang = $state<LessonLanguage>('en');
  let currentLessonId = $state<string | null>(null);
  let lessonNavigation = $derived(lessonResultNavigation(courseModules, currentLessonId));

  // Weak Keys
  let weakKeysData = $state<Array<{ ch: string; error_count: number; accuracy: number; rank: number }>>([]);
  let weakKeysCharStats = $state<Record<string, { correct: number; incorrect: number; total: number }>>({});

  // Dashboard
  let dashboardStats = $state<DashboardStatsResponse | null>(null);
  let weeklySummaries = $state<WeeklySummaryResponse[] | null>(null);

  async function loadHistory(page = 0) {
    const r = await ipc.getStatsHistory(20, page * 20);
    history = r.tests;
    historyTotal = r.total;
    historyPage = page;
  }

  // Лёгкое обновление только счётчика (бейдж навигации) без перезагрузки списка.
  async function loadHistoryTotal() {
    try {
      const r = await ipc.getStatsHistory(1, 0);
      historyTotal = r.total;
    } catch {
      // Best-effort: бейдж обновится при следующем визите History.
    }
  }

  function historyPrevPage() {
    if (historyPage > 0) void loadHistory(historyPage - 1);
  }

  function historyNextPage() {
    if ((historyPage + 1) * 20 < historyTotal) void loadHistory(historyPage + 1);
  }

  async function loadBests() {
    bests = await ipc.getPersonalBests();
  }

  async function loadCustomTexts() {
    customTexts = await ipc.getCustomTexts(50);
  }

  function openEditor(ct: CustomText | null, fallbackLanguage: LanguageCode) {
    editingText = ct;
    newName = ct ? ct.name : '';
    newTextContent = ct ? ct.text : '';
    customTextLanguage = ct?.language ?? fallbackLanguage;
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
      deps.setError(`Save error: ${err}`);
    }
  }

  async function deleteCustomText(id: number) {
    await ipc.deleteCustomText(id);
    await loadCustomTexts();
  }

  async function searchCustom(q: string) {
    searchText = q;
    if (q.trim()) {
      customTexts = await ipc.searchCustomTexts(q, 20);
    } else {
      await loadCustomTexts();
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
      deps.setError(`Lessons error: ${e}`);
    }
  }

  function applyLessonCompletion(lessonId: string, passed: boolean, wpm: number, accuracy: number) {
    lessonProgress = {
      ...lessonProgress,
      [lessonId]: {
        status: passed
          ? 'completed'
          : (lessonProgress[lessonId]?.status === 'completed' ? 'completed' : 'in_progress'),
        best_wpm: Math.max(lessonProgress[lessonId]?.best_wpm ?? 0, wpm),
        best_accuracy: Math.max(lessonProgress[lessonId]?.best_accuracy ?? 0, accuracy),
      },
    };
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
      deps.setError(`Weak keys error: ${e}`);
    }
  }

  async function loadDashboard() {
    try {
      dashboardStats = await ipc.getDashboardStats();
      // Weekly summaries are best-effort: without them the dashboard
      // simply renders without the weekly strip.
      try {
        weeklySummaries = await ipc.getWeeklySummaries();
      } catch {
        weeklySummaries = null;
      }
      // Weak-keys для виджета «Тренировка дня» — грузим и здесь, иначе
      // при первом визите на дашборд виджет никогда не появляется
      // (раньше weakKeysData заполнялся только на weakkeys-вью).
      try {
        const data = await ipc.analyzeWeakKeys();
        weakKeysData = data.weak_keys || [];
      } catch {
        // Best-effort: без weak-keys дашборд просто без виджета.
      }
    } catch (e) {
      deps.setError(`Dashboard error: ${e}`);
    }
  }

  return {
    get history() { return history; },
    get historyTotal() { return historyTotal; },
    get historyPage() { return historyPage; },
    get bests() { return bests; },
    get customTexts() { return customTexts; },
    get editingText() { return editingText; },
    get newName() { return newName; },
    get newTextContent() { return newTextContent; },
    get customTextLanguage() { return customTextLanguage; },
    get showEditor() { return showEditor; },
    get searchText() { return searchText; },
    get courseModules() { return courseModules; },
    get lessonProgress() { return lessonProgress; },
    get lessonLang() { return lessonLang; },
    get currentLessonId() { return currentLessonId; },
    get lessonNavigation() { return lessonNavigation; },
    get weakKeysData() { return weakKeysData; },
    get weakKeysCharStats() { return weakKeysCharStats; },
    get dashboardStats() { return dashboardStats; },
    get weeklySummaries() { return weeklySummaries; },
    set newName(value: string) { newName = value; },
    set newTextContent(value: string) { newTextContent = value; },
    set customTextLanguage(value: LanguageCode) { customTextLanguage = value; },
    set showEditor(value: boolean) { showEditor = value; },
    set lessonLang(value: LessonLanguage) { lessonLang = value; },
    set currentLessonId(value: string | null) { currentLessonId = value; },
    loadHistory,
    loadHistoryTotal,
    historyPrevPage,
    historyNextPage,
    loadBests,
    loadCustomTexts,
    openEditor,
    saveCustomText,
    deleteCustomText,
    searchCustom,
    loadLessons,
    applyLessonCompletion,
    loadWeakKeys,
    loadDashboard,
  };
}
