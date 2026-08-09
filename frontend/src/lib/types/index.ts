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
  session_state: SessionState;
  key_result: string;
  caret_pos: number;
  visible_pos: { row: number; col: number };
  live_stats: { wpm: number; raw_wpm: number; accuracy: number; elapsed_ms: number } | null;
  test_complete: FinalStats | null;
}

export type SessionState = 'idle' | 'running' | 'awaiting_persistence' | 'persisting' | 'persisted';

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
  session_id: string;
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
  has_replay: boolean;
}

export interface ReplayFrame {
  frame_index: number;
  timestamp_ms: number;
  position: number;
  expected_char: string;
  typed_char: string | null;
  correct: boolean;
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
  language: LanguageCode;
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
  blind_mode_enabled: boolean;
  ui_language: string;
  vim_mode: boolean;
  daily_goal_type: string;
  daily_goal_wpm: number;
  daily_goal_accuracy: number;
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

export type ViewName = 'dashboard' | 'test' | 'history' | 'bests' | 'custom' | 'settings' | 'lessons' | 'weakkeys' | 'analytics' | 'achievements';
export type ModeName = 'time' | 'words' | 'quote' | 'custom';
export type LanguageCode = 'en' | 'ru' | 'de' | 'uk' | 'cs' | 'pl' | 'ro' | 'it' | 'fr' | 'es' | 'pt' | 'ja' | 'zh-hk' | 'zh-tw' | 'ko';

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

export interface LessonProgressRecord {
  id: number;
  lesson_id: string;
  module_id: string;
  language: LanguageCode;
  difficulty: string;
  status: string;
  best_wpm: number;
  best_accuracy: number;
  attempts: number;
  last_attempt_at: string | null;
  completed_at: string | null;
}

export interface WeakKey {
  ch: string;
  error_count: number;
  total: number;
  accuracy: number;
  rank: number;
}

export interface WeakKeysReport {
  weak_keys: WeakKey[];
  total_chars_analyzed: number;
  overall_accuracy: number;
  critical_count: number;
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  unlocked: boolean;
  unlocked_at: string | null;
}

export interface Insight {
  level: string;
  title: string;
  message: string;
}

export interface ConsistencyReport {
  score: number;
  mean_wpm: number;
  std_dev: number;
  cv: number;
  samples: number;
}

export interface SoundOutputResponse {
  frequency: number;
  duration_ms: number;
  volume: number;
  event: string;
}

export interface DashboardStatsResponse {
  current_streak: number;
  longest_streak: number;
  avg_wpm: number;
  avg_accuracy: number;
  tests_today: number;
  tests_this_week: number;
  total_tests: number;
  daily_goal_met: boolean;
}

export interface ProgressPoint {
  date: string;
  wpm: number;
  accuracy: number;
  tests: number;
}

export type ProfileImportPolicy = 'merge' | 'replace';

export interface CollectionImportPlan {
  incoming: number;
  existing: number;
  to_insert: number;
}

export interface ProfileImportPlan {
  policy: ProfileImportPolicy;
  tests: CollectionImportPlan;
  personal_bests: CollectionImportPlan;
  daily_stats: CollectionImportPlan;
  streaks: CollectionImportPlan;
  custom_texts: CollectionImportPlan;
  lesson_progress: CollectionImportPlan;
}
