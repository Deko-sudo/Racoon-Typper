-- V003__replays.sql — таблица для replay данных

CREATE TABLE IF NOT EXISTS test_replays (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    test_id         INTEGER NOT NULL,
    frame_index     INTEGER NOT NULL,
    timestamp_ms    INTEGER NOT NULL,
    position        INTEGER NOT NULL,
    expected_char   TEXT NOT NULL,
    typed_char      TEXT,
    correct         BOOLEAN NOT NULL DEFAULT 1,
    FOREIGN KEY (test_id) REFERENCES tests(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_replays_test_id ON test_replays(test_id);
CREATE INDEX IF NOT EXISTS idx_replays_frame ON test_replays(test_id, frame_index);