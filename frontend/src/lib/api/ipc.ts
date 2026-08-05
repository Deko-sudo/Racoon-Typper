// IPC wrappers — typed invoke calls.

import { invoke } from '@tauri-apps/api/core';
import type {
  AppSettings,
  Achievement,
  ConsistencyReport,
  CourseResponse,
  CustomText,
  DashboardStatsResponse,
  EngineOutput,
  Insight,
  LessonProgressRecord,
  PersonalBest,
  ProfileImportPlan,
  ProfileImportPolicy,
  ProgressPoint,
  ReplayFrame,
  StatsHistoryResponse,
  TestSessionResponse,
  ThemeInfo,
  SoundOutputResponse,
  WeakKeysReport,
} from '../types/index';

export interface IpcError {
  code?: string;
  message?: string;
}

export function ipcErrorMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null) {
    const typed = error as IpcError;
    if (typed.message) return typed.message;
    if (typed.code) return typed.code;
  }
  if (error instanceof Error) return error.message;
  return String(error);
}

export async function startTest(params: {
  mode: string;
  language: string;
  duration?: number;
  wordCount?: number;
  quoteId?: number;
  text?: string;
}): Promise<TestSessionResponse> {
  return invoke<TestSessionResponse>('start_test', params);
}

export async function processKey(key: string, code: string, sessionId: string): Promise<EngineOutput> {
  return invoke<EngineOutput>('process_key', { key, code, sessionId });
}

export async function abortSession(sessionId: string): Promise<void> {
  return invoke('abort_session', { sessionId });
}

export async function getStatsHistory(limit: number, offset = 0): Promise<StatsHistoryResponse> {
  return invoke<StatsHistoryResponse>('get_stats_history', { limit, offset });
}

export async function getPersonalBests(): Promise<PersonalBest[]> {
  return invoke<PersonalBest[]>('get_personal_bests', {});
}

export async function getCustomTexts(limit = 50): Promise<CustomText[]> {
  return invoke<CustomText[]>('get_custom_texts', { limit });
}

export async function saveCustomText(name: string, text: string, language: string): Promise<number> {
  return invoke<number>('save_custom_text', { name, text, language });
}

export async function updateCustomText(id: number, name: string, text: string, language: string): Promise<void> {
  return invoke('update_custom_text', { id, name, text, language });
}

export async function deleteCustomText(id: number): Promise<void> {
  return invoke('delete_custom_text', { id });
}

export async function searchCustomTexts(query: string, limit = 20): Promise<CustomText[]> {
  return invoke<CustomText[]>('search_custom_texts', { query, limit });
}

export async function startCustomTextTest(customTextId: number): Promise<TestSessionResponse> {
  return invoke<TestSessionResponse>('start_custom_text_test', { customTextId });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings');
}

export async function setSetting(key: string, value: unknown): Promise<AppSettings> {
  return invoke<AppSettings>('set_setting', { key, value });
}

export async function getThemes(): Promise<ThemeInfo[]> {
  return invoke<ThemeInfo[]>('get_themes');
}

export async function getThemeCss(name: string): Promise<string> {
  return invoke<string>('get_theme_css', { name });
}

// Lessons
export async function getCourse(language: string): Promise<CourseResponse> {
  return invoke<CourseResponse>('get_course', { language });
}

export async function getLessonProgress(language: string): Promise<LessonProgressRecord[]> {
  return invoke<LessonProgressRecord[]>('get_lesson_progress', { language });
}

export async function startLesson(lessonId: string, language: string): Promise<TestSessionResponse> {
  return invoke<TestSessionResponse>('start_lesson', { lessonId, language });
}

// Weak Keys
export async function analyzeWeakKeys(): Promise<WeakKeysReport> {
  return invoke<WeakKeysReport>('analyze_weak_keys');
}

export async function generateWeakKeysTraining(language: string, wordCount?: number): Promise<string> {
  return invoke<string>('generate_weak_keys_training', { language, wordCount });
}

// Dashboard
export async function getDashboardStats(): Promise<DashboardStatsResponse> {
  return invoke<DashboardStatsResponse>('get_dashboard_stats');
}

export async function getProgressHistory(days?: number): Promise<ProgressPoint[]> {
  return invoke<ProgressPoint[]>('get_progress_history', { days });
}

// Analytics
export async function getAchievements(): Promise<Achievement[][]> {
  return invoke<Achievement[][]>('get_achievements');
}

export async function getInsights(): Promise<Insight[][]> {
  return invoke<Insight[][]>('get_insights');
}

export async function getConsistency(): Promise<ConsistencyReport> {
  return invoke<ConsistencyReport>('get_consistency');
}

export async function exportData(format: 'json' | 'csv'): Promise<string> {
  return invoke<string>('export_data', { format });
}

// Versioned portable profile transfer. `replace` is destructive for portable
// profile tables, so callers should always display the preview before applying it.
export async function exportProfile(): Promise<string> {
  return invoke<string>('export_profile');
}

export async function previewProfileImport(
  document: string,
  policy: ProfileImportPolicy,
): Promise<ProfileImportPlan> {
  return invoke<ProfileImportPlan>('preview_profile_import', { document, policy });
}

export async function importProfile(
  document: string,
  policy: ProfileImportPolicy,
): Promise<ProfileImportPlan> {
  return invoke<ProfileImportPlan>('import_profile', { document, policy });
}

// Replay
export async function getReplay(testId: number): Promise<ReplayFrame[]> {
  return invoke<ReplayFrame[]>('get_replay', { testId });
}

// Sound
export async function getSoundEvent(event: string): Promise<SoundOutputResponse | null> {
  return invoke<SoundOutputResponse | null>('get_sound_event', { event });
}
