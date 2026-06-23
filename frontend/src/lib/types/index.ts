// Shared TypeScript types for frontend — mirror of domain types.

export interface CharStatus {
  expected: string;
  typed: string | null;
  status: 'pending' | 'correct' | 'incorrect' | 'backspaced';
}

export interface TestSessionResponse {
  session_id: string;
  text: string;
  text_length: number;
  mode_type: string;
  mode_config: Record<string, unknown>;
  language: string;
}

export interface EngineOutput {
  key_result: string;
  caret_pos: number;
  visible_pos: { row: number; col: number };
  live_stats: { wpm: number; raw_wpm: number; accuracy: number; elapsed_ms: number } | null;
  test_complete: FinalStats | null;
}

export interface FinalStats {
  wpm: number;
  raw_wpm: number;
  accuracy: number;
  raw_accuracy: number;
  consistency: number | null;
  correct_chars: number;
  incorrect_chars: number;
  backspaces: number;
  char_stats: Record<string, { correct: number; incorrect: number; total: number }>;
  heatmap: Record<string, { total_attempts: number; correct: number; incorrect: number; avg_wpm_at_key: number }>;
  graph_data: unknown | null;
  duration_ms: number;
}

export interface TestSummary {
  id: number;
  created_at: string;
  mode_type: string;
  mode_config: Record<string, unknown>;
  language: string;
  wpm: number;
  raw_wpm: number;
  accuracy: number;
  raw_accuracy: number;
  consistency: number | null;
  duration_ms: number;
  is_pb: boolean;
}

export interface TestDetail {
  id: number;
  created_at: string;
  mode_type: string;
  mode_config: Record<string, unknown>;
  language: string;
  wpm: number;
  raw_wpm: number;
  accuracy: number;
  raw_accuracy: number;
  duration_ms: number;
  char_stats: Record<string, unknown>;
  heatmap_data: Record<string, unknown>;
}

export interface StatsHistoryResponse {
  tests: TestSummary[];
  total: number;
}

export interface PersonalBest {
  mode_type: string;
  mode_config: Record<string, unknown>;
  best_wpm: number;
  best_wpm_test_id: number | null;
  best_accuracy: number;
  best_accuracy_test_id: number | null;
  best_consistency: number | null;
  best_consistency_test_id: number | null;
  updated_at: string;
}

export interface CustomText {
  id: number;
  name: string;
  text: string;
  created_at: string;
  last_used_at: string | null;
  use_count: number;
}

export interface AppSettings {
  theme: string;
  font_size: number;
  caret_style: string;
  show_live_wpm: boolean;
  show_accuracy: boolean;
  show_keyboard_trainer: boolean;
  show_hand_guide: boolean;
  show_layout_warnings: boolean;
  show_capslock_warnings: boolean;
  sound_enabled: boolean;
  sound_volume: number;
  zen_mode_enabled: boolean;
}

export interface ThemeInfo {
  name: string;
  display_name: string;
  is_dark: boolean;
  preview_colors: {
    bg: string;
    main: string;
    text: string;
    error: string;
  };
}

export type ViewName = 'dashboard' | 'test' | 'history' | 'bests' | 'custom' | 'settings' | 'lessons' | 'weakkeys' | 'analytics';
export type ModeName = 'time' | 'words' | 'quote' | 'custom';
export type LanguageCode = 'en' | 'ru';

export interface ModuleResponse {
  id: string;
  name: string;
  difficulty: string;
  order: number;
  lessons: LessonResponse[];
}

export interface LessonResponse {
  id: string;
  name: string;
  text_length: number;
}

export interface CourseResponse {
  language: string;
  modules: ModuleResponse[];
}

export interface DashboardStatsResponse {
  current_streak: number;
  longest_streak: number;
  avg_wpm: number;
  avg_accuracy: number;
  tests_today: number;
  tests_this_week: number;
  total_tests: number;
}

export interface StreakInfoResponse {
  current: number;
  longest: number;
  is_active: boolean;
}

export interface ProgressPoint {
  date: string;
  wpm: number;
  accuracy: number;
  tests: number;
}