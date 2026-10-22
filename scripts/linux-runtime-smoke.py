#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
"""Isolated Linux startup/restart smoke; no claim of typed-input or GUI coverage."""
import argparse
from contextlib import closing
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import tempfile
import time

MARKER = ("Linux package persistence probe", "фыва asdf", "ru")


def database_ready(path):
    try:
        with closing(sqlite3.connect(f"{path.as_uri()}?mode=ro", uri=True, timeout=0.2)) as db:
            return db.execute("SELECT count(*) FROM refinery_schema_history").fetchone()[0] == 9
    except sqlite3.Error:
        return False


def check_database(path):
    with closing(sqlite3.connect(f"{path.as_uri()}?mode=ro", uri=True)) as db:
        if db.execute("PRAGMA integrity_check").fetchall() != [("ok",)]:
            raise RuntimeError("SQLite integrity check failed")
        if db.execute("PRAGMA foreign_key_check").fetchall():
            raise RuntimeError("SQLite foreign-key violations")
        row = db.execute("SELECT name, text, language FROM custom_texts WHERE name=?", (MARKER[0],)).fetchall()
        if row != [MARKER]:
            raise RuntimeError("Restart lost or altered the persisted custom text")


def run_smoke(command, workspace, timeout=45):
    workspace = workspace.resolve()
    workspace.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ)
    for key, suffix in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"), ("XDG_CACHE_HOME", "cache")]:
        directory = workspace / suffix
        directory.mkdir(exist_ok=True)
        env[key] = str(directory)
    path = workspace / "data/com.racoon.typper/data.db"
    if path.exists():
        raise RuntimeError("Smoke requires a NEW disposable profile")
    for attempt in range(2):
        with (workspace / f"launch-{attempt+1}.log").open("wb") as log:
            process = subprocess.Popen(command, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                deadline = time.monotonic() + timeout
                while not database_ready(path):
                    if process.poll() is not None:
                        raise RuntimeError(f"Application exited before database initialization; see {log.name}")
                    if time.monotonic() >= deadline:
                        raise RuntimeError(f"Database initialization timed out; see {log.name}")
                    time.sleep(0.1)
                # On restart, wait for startup to run instead of accepting the
                # database left over from the first process as proof of launch.
                time.sleep(1)
                if process.poll() is not None:
                    raise RuntimeError(f"Application exited during startup; see {log.name}")
            finally:
                # Only our new process session, never all applications matching
                # an executable name. The shell wrappers no longer use pkill.
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=5)
        if attempt == 0:
            with sqlite3.connect(path) as db:
                db.execute("INSERT INTO custom_texts(name,text,language,created_at,use_count) VALUES (?,?,?,'2026-01-01T00:00:00Z',0)", MARKER)
        check_database(path)
    return {"startup_runs": 2, "persistent_custom_text": True, "integrity": "ok", "foreign_key_violations": 0, "profile": str(workspace), "gui_and_typed_input_verified": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("a command after -- is required")
    if args.workspace:
        print(json.dumps(run_smoke(command, args.workspace)))
    else:
        with tempfile.TemporaryDirectory(prefix="racoon-linux-smoke-") as directory:
            print(json.dumps(run_smoke(command, Path(directory))))
