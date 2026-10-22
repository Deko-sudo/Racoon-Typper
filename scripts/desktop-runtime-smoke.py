#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
"""CI-only Windows/macOS window + SQLite restart smoke; not a typing/render test."""
import argparse
from contextlib import closing
import ctypes
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import sqlite3
import subprocess
import sys
import time

spec = importlib.util.spec_from_file_location("sqlite_smoke", Path(__file__).with_name("linux-runtime-smoke.py"))
checks = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checks)
checks.MARKER = ("Desktop package persistence probe", "фыва asdf", "ru")


def profile_directory():
    if sys.platform == "win32":
        buffer = ctypes.create_unicode_buffer(32768)
        if ctypes.windll.shell32.SHGetFolderPathW(None, 26, None, 0, buffer) != 0:
            raise RuntimeError("Cannot resolve the Windows roaming application directory")
        return Path(buffer.value) / "com.racoon.typper"
    if sys.platform == "darwin":
        return Path.home() / "Library/Application Support/com.racoon.typper"
    raise RuntimeError("This smoke only supports Windows/macOS CI runners")


def visible_window(pid, probe):
    if sys.platform == "darwin":
        return subprocess.run([str(probe), str(pid)], check=False, timeout=10).returncode == 0
    from ctypes import wintypes
    user32 = ctypes.windll.user32
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    user32.IsWindowVisible.argtypes = [wintypes.HWND]
    user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user32.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
    user32.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
    found = []

    @callback_type
    def visit(handle, _):
        owner = wintypes.DWORD()
        rect = wintypes.RECT()
        user32.GetWindowThreadProcessId(handle, ctypes.byref(owner))
        if owner.value == pid and user32.IsWindowVisible(handle) and user32.GetWindowRect(handle, ctypes.byref(rect)):
            if rect.right - rect.left >= 400 and rect.bottom - rect.top >= 300:
                found.append(True)
        return True

    user32.EnumWindows(visit, 0)
    return bool(found)


def stop_owned_process(process):
    if process.poll() is not None:
        return
    if os.name == "nt":
        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], check=True, timeout=20)
    else:
        os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


def run_smoke(command, profile, evidence, window_check, timeout=120, interact=None):
    # Never replace, seed or remove an existing real user profile. These OSes
    # use known folders, so changing APPDATA/XDG_* would not isolate Tauri.
    evidence.mkdir(parents=True, exist_ok=True)
    profile.mkdir(parents=True, exist_ok=False)
    database = profile / "data.db"
    try:
        (profile / "settings.toml").write_text("onboarding_completed = true\n", encoding="utf-8")
        for attempt in range(2):
            with (evidence / f"launch-{attempt+1}.log").open("wb") as log:
                process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT, start_new_session=os.name != "nt")
                try:
                    deadline = time.monotonic() + timeout
                    while not (checks.database_ready(database) and window_check(process.pid)):
                        if process.poll() is not None:
                            raise RuntimeError(f"Launch {attempt+1} exited with {process.returncode}")
                        if time.monotonic() >= deadline:
                            raise RuntimeError(f"Launch {attempt+1} did not initialize a database AND visible native window")
                        time.sleep(0.2)
                    time.sleep(1)
                    if process.poll() is not None:
                        raise RuntimeError("Application exited immediately after startup")
                    if attempt == 1 and interact:
                        interact(process.pid)
                finally:
                    stop_owned_process(process)
            if attempt == 0:
                with closing(sqlite3.connect(database)) as db, db:
                    db.execute("INSERT INTO custom_texts(name,text,language,created_at,use_count) VALUES (?,?,?,'2026-01-01T00:00:00Z',0)", checks.MARKER)
            checks.check_database(database)
        if interact:
            # The only direct INSERT above is a custom text, never a result.
            with closing(sqlite3.connect(database)) as db:
                rows = db.execute("SELECT language, correct_chars, incorrect_chars FROM tests").fetchall()
            if rows != [("en", 4, 0), ("en", 4, 0)]:
                raise RuntimeError("Native UI did not persist two exact four-character results")
        result = {"startup_runs": 2, "native_window_observed": True, "persistent_custom_text": True,
                  "integrity": "ok", "foreign_key_violations": 0, "typed_input_png_clipboard_verified": False}
        if interact:
            result["native_ui"] = "two EN completions, repeat, abort, PNG file and OS image clipboard; hardware DPI/paste unverified"
        (evidence / "runtime-smoke.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        return result
    finally:
        shutil.rmtree(profile)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--executable", required=True, type=Path)
    parser.add_argument("--evidence", required=True, type=Path)
    parser.add_argument("--window-probe", type=Path)
    parser.add_argument("--ui-probe", type=Path, help="Optional native UI investigation; failures are not a UI pass")
    args = parser.parse_args()
    # Refuse use on a personal desktop, even when no profile exists yet.
    if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("RUNNER_ENVIRONMENT") != "github-hosted":
        parser.error("Use only on a fresh GitHub-hosted CI runner, never a personal profile")
    if sys.platform == "darwin" and not args.window_probe:
        parser.error("macOS requires a compiled --window-probe")
    interact = None
    if args.ui_probe:
        checks.MARKER = ("Desktop package persistence probe", "asdf", "en")
        def interact(pid):
            probe = str(args.ui_probe.resolve())
            evidence = str(args.evidence.resolve())
            command = (["powershell.exe", "-NoProfile", "-File", probe, "-AppProcessId", str(pid), "-Evidence", evidence]
                       if sys.platform == "win32" else [probe, str(pid), evidence])
            subprocess.run(command, check=True, timeout=150)
    print(json.dumps(run_smoke([str(args.executable.resolve())], profile_directory(), args.evidence,
                               lambda pid: visible_window(pid, args.window_probe), interact=interact)))
