-- V001__initial.sql — начальная схема БД для Racoon Typper v0.1.0
-- 6 таблиц: tests, personal_bests, lesson_progress, daily_stats, streaks, custom_texts

-- tests: история всех completed тестов
CREATE TABLE IF NOT EXISTS tests (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at      TEXT NOT NULL,
    mode_type        TEXT NOT NULL,
    mode_config      TEXT NOT NULL,
    language        TEXT NOT NULL,
    text_length      INTEGER NOT NULL,
    duration_ms     INTEGER NOT NULL,
    wpm             REAL NOT NULL,
    raw_wpm         REAL NOT NULL,
    accuracy        REAL NOT NULL,
    raw_accuracy    REAL NOT NULL,
    consistency     REAL,
    correct_chars   INTEGER NOT NULL,
    incorrect_chars INTEGER NOT NULL,
    backspaces      INTEGER NOT NULL,
    char_stats       TEXT NOT NULL,
    heatmap_data     TEXT NOT NULL,
    graph_data       TEXT,
    is_pb           BOOLEAN NOT NULL DEFAULT 0,
    tags            TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_tests_created_at ON tests(created_at);
CREATE INDEX IF NOT EXISTS idx_tests_mode_config ON tests(mode_type, mode_config);
CREATE INDEX IF NOT EXISTS idx_tests_wpm ON tests(wpm);

-- personal_bests: личные рекорды по конфигурациям режимов
CREATE TABLE IF NOT EXISTS personal_bests (
    id                        INTEGER PRIMARY KEY AUTOINCREMENT,
    mode_type                 TEXT NOT NULL,
    mode_config_hash          TEXT NOT NULL,
    mode_config               TEXT NOT NULL,
    best_wpm                  REAL NOT NULL,
    best_wpm_test_id          INTEGER REFERENCES tests(id),
    best_accuracy             REAL NOT NULL,
    best_accuracy_test_id     INTEGER REFERENCES tests(id),
    best_consistency          REAL,
    best_consistency_test_id  INTEGER REFERENCES tests(id),
    updated_at                TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uniq_pb_mode_config_hash ON personal_bests(mode_type, mode_config_hash);

-- lesson_progress: прогресс по урокам
CREATE TABLE IF NOT EXISTS lesson_progress (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    lesson_id           TEXT NOT NULL,
    module_id           TEXT NOT NULL,
    difficulty          TEXT NOT NULL DEFAULT 'beginner',
    status              TEXT NOT NULL DEFAULT 'not_started',
    attempts            INTEGER NOT NULL DEFAULT 0,
    best_wpm            REAL NOT NULL DEFAULT 0,
    best_accuracy       REAL NOT NULL DEFAULT 0,
    last_wpm            REAL,
    last_accuracy       REAL,
    last_attempt_at    TEXT,
    completed_at        TEXT,
    exercises_completed INTEGER NOT NULL DEFAULT 0,
    total_exercises     INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_lesson_progress_lesson_id ON lesson_progress(lesson_id);
CREATE INDEX IF NOT EXISTS idx_lesson_progress_module_id ON lesson_progress(module_id);
CREATE INDEX IF NOT EXISTS idx_lesson_progress_difficulty ON lesson_progress(difficulty);

-- daily_stats: агрегированная статистика по дням
CREATE TABLE IF NOT EXISTS daily_stats (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    date              TEXT NOT NULL,
    total_tests       INTEGER NOT NULL DEFAULT 0,
    total_time_ms     INTEGER NOT NULL DEFAULT 0,
    total_chars       INTEGER NOT NULL DEFAULT 0,
    best_wpm          REAL NOT NULL DEFAULT 0,
    avg_wpm           REAL NOT NULL DEFAULT 0,
    avg_accuracy      REAL NOT NULL DEFAULT 0,
    lessons_completed INTEGER NOT NULL DEFAULT 0,
    daily_goal_met    BOOLEAN NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_stats(date);

-- streaks: серии
CREATE TABLE IF NOT EXISTS streaks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    type            TEXT NOT NULL,
    current_streak  INTEGER NOT NULL DEFAULT 0,
    longest_streak  INTEGER NOT NULL DEFAULT 0,
    last_date       TEXT,
    started_date    TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_streaks_type ON streaks(type);

-- custom_texts: сохранённые пользовательские тексты
CREATE TABLE IF NOT EXISTS custom_texts (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT NOT NULL,
    text         TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    last_used_at TEXT,
    use_count    INTEGER NOT NULL DEFAULT 0
);