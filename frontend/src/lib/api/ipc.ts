// IPC wrappers — typed invoke calls.

import { invoke } from '@tauri-apps/api/core';
import type {
  AppSettings,
  CourseResponse,
  CustomText,
  DashboardStatsResponse,
  FinalStats,
  PersonalBest,
  ProgressPoint,
  StatsHistoryResponse,
  StreakInfoResponse,
  TestDetail,
  TestSessionResponse,
  ThemeInfo,
} from '../types/index';

export async function ping(): Promise<string> {
  return invoke<string>('ping');
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

export async function processKey(key: string, code: string) {
  return invoke<{ key_result: string; caret_pos: number; live_stats: { wpm: number; accuracy: number; elapsed_ms: number } | null; test_complete: FinalStats | null }>(
    'process_key',
    { key, code }
  );
}

export async function abortSession(): Promise<void> {
  return invoke('abort_session');
}

export async function getStatsHistory(limit: number, offset = 0): Promise<StatsHistoryResponse> {
  return invoke<StatsHistoryResponse>('get_stats_history', { limit, offset });
}

export async function getTestDetail(id: number): Promise<TestDetail> {
  return invoke<TestDetail>('get_test_detail', { id });
}

export async function getPersonalBests(): Promise<PersonalBest[]> {
  return invoke<PersonalBest[]>('get_personal_bests', {});
}

export async function getCustomTexts(limit = 50): Promise<CustomText[]> {
  return invoke<CustomText[]>('get_custom_texts', { limit });
}

export async function saveCustomText(name: string, text: string): Promise<number> {
  return invoke<number>('save_custom_text', { name, text });
}

export async function updateCustomText(id: number, name: string, text: string): Promise<void> {
  return invoke('update_custom_text', { id, name, text });
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

export async function getLessonProgress(language: string): Promise<unknown> {
  return invoke('get_lesson_progress', { language });
}

export async function startLesson(lessonId: string, language: string): Promise<TestSessionResponse> {
  return invoke<TestSessionResponse>('start_lesson', { lessonId, language });
}

export async function completeLesson(lessonId: string, wpm: number, accuracy: number): Promise<void> {
  return invoke('complete_lesson', { lessonId, wpm, accuracy });
}

// Weak Keys
export async function analyzeWeakKeys(): Promise<unknown> {
  return invoke('analyze_weak_keys');
}

export async function generateWeakKeysTraining(language: string, wordCount?: number): Promise<string> {
  return invoke<string>('generate_weak_keys_training', { language, wordCount });
}

// Dashboard
export async function getDashboardStats(): Promise<DashboardStatsResponse> {
  return invoke<DashboardStatsResponse>('get_dashboard_stats');
}

export async function getStreakInfo(): Promise<StreakInfoResponse> {
  return invoke<StreakInfoResponse>('get_streak_info');
}

export async function getProgressHistory(days?: number): Promise<ProgressPoint[]> {
  return invoke<ProgressPoint[]>('get_progress_history', { days });
}

// Analytics
export async function getAchievements(): Promise<unknown> {
  return invoke('get_achievements');
}

export async function getInsights(): Promise<unknown> {
  return invoke('get_insights');
}

export async function getConsistency(): Promise<unknown> {
  return invoke('get_consistency');
}

export async function exportData(format: 'json' | 'csv'): Promise<string> {
  return invoke<string>('export_data', { format });
}

// Replay
export async function getReplay(testId: number): Promise<unknown> {
  return invoke('get_replay', { testId });
}

export async function hasReplay(testId: number): Promise<boolean> {
  return invoke<boolean>('has_replay', { testId });
}

// Sound
export async function getSoundEvent(event: string): Promise<{ frequency: number; duration_ms: number; volume: number; event: string } | null> {
  return invoke('get_sound_event', { event });
}

// Session Recovery
export async function saveSessionState(session: { text: string; typed_chars: boolean[]; mode_type: string; language: string; elapsed_ms: number; saved_at: string }): Promise<void> {
  return invoke('save_session_state', { session });
}

export async function loadSessionState(): Promise<{ text: string; typed_chars: boolean[]; mode_type: string; language: string; elapsed_ms: number; saved_at: string } | null> {
  return invoke('load_session_state');
}

export async function clearSessionState(): Promise<void> {
  return invoke('clear_session_state');
}

// Extended Stats
export async function getExtendedStats(): Promise<{ best_day_wpm: number; best_day_date: string; most_active_hour: number; avg_session_duration_ms: number; total_chars: number; total_words: number }> {
  return invoke('get_extended_stats');
}

// Profile Export
export async function exportProfile(): Promise<string> {
  return invoke<string>('export_profile');
}