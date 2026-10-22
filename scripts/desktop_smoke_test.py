# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
import importlib.util
from contextlib import closing
import os
from pathlib import Path
import sqlite3
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).with_name("desktop-runtime-smoke.py")
SPEC = importlib.util.spec_from_file_location("desktop_smoke", SCRIPT)
SMOKE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SMOKE)
FAKE_APP = '''import os, sqlite3, time
from pathlib import Path
p=Path(os.environ['SMOKE_PROFILE'])/'data.db'
if os.environ.get('SMOKE_LOSE_DATA') == '1': p.unlink(missing_ok=True)
with sqlite3.connect(p) as db:
 db.execute('CREATE TABLE IF NOT EXISTS refinery_schema_history(version INTEGER)')
 if not db.execute('SELECT count(*) FROM refinery_schema_history').fetchone()[0]: db.executemany('INSERT INTO refinery_schema_history VALUES (?)', [(n,) for n in range(1,10)])
 db.execute('CREATE TABLE IF NOT EXISTS custom_texts(name TEXT,text TEXT,language TEXT,created_at TEXT,use_count INTEGER)')
while True: time.sleep(.1)
'''


class DesktopSmokeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.profile = self.directory / "profile"
        self.evidence = self.directory / "evidence"
        self.app = self.directory / "fake.py"
        self.app.write_text(FAKE_APP, encoding="utf-8")
        self.patch = patch.dict(os.environ, {"SMOKE_PROFILE": str(self.profile)})
        self.patch.start()
        self.addCleanup(self.patch.stop)

    def run_app(self, **kwargs):
        return SMOKE.run_smoke([sys.executable, str(self.app)], self.profile, self.evidence, **kwargs)

    def test_preserves_marker_and_observes_window_on_each_launch(self):
        pids = set()
        result = self.run_app(window_check=lambda pid: pids.add(pid) or True)
        self.assertEqual(len(pids), 2)
        self.assertTrue(result['persistent_custom_text'])
        self.assertFalse(result['typed_input_png_clipboard_verified'])
        self.assertFalse(self.profile.exists())
        self.assertTrue((self.evidence/'runtime-smoke.json').exists())

    def test_sqlite_handles_are_closed_before_profile_removal(self):
        connections = []
        original = sqlite3.connect

        class Tracked(sqlite3.Connection):
            closed = False

            def close(self):
                self.closed = True
                super().close()

        def connect(*args, **kwargs):
            connection = original(*args, **kwargs, factory=Tracked)
            connections.append(connection)
            return connection

        with patch.object(sqlite3, 'connect', side_effect=connect):
            self.run_app(window_check=lambda pid: True)
        self.assertTrue(connections)
        self.assertTrue(all(connection.closed for connection in connections), 'SQLite context managers commit but do not close; Windows cannot delete these open files')

    def test_database_alone_cannot_pass_without_a_window(self):
        with self.assertRaisesRegex(RuntimeError, "visible native window"):
            self.run_app(window_check=lambda pid: False, timeout=.5)
        self.assertFalse(self.profile.exists())
        self.assertFalse((self.evidence/'runtime-smoke.json').exists())

    def test_restart_data_loss_fails_even_with_a_window(self):
        with patch.dict(os.environ, {'SMOKE_LOSE_DATA': '1'}):
            with self.assertRaisesRegex(RuntimeError, "lost or altered"):
                self.run_app(window_check=lambda pid: True)

    def test_existing_profile_is_never_seeded_or_removed(self):
        self.profile.mkdir()
        marker = self.profile / "personal-data"
        marker.write_bytes(b"keep me")
        with self.assertRaises(FileExistsError):
            self.run_app(window_check=lambda pid: True)
        self.assertEqual(marker.read_bytes(), b"keep me")
        self.assertFalse((self.profile/'settings.toml').exists())

    def test_unwritable_evidence_does_not_leave_a_partial_profile(self):
        self.evidence.write_text('file, not directory', encoding='utf-8')
        with self.assertRaises(FileExistsError):
            self.run_app(window_check=lambda pid: True)
        self.assertFalse(self.profile.exists())

    def test_startup_exit_is_not_a_smoke_success(self):
        with self.assertRaisesRegex(RuntimeError, "exited"):
            SMOKE.run_smoke([sys.executable, '-c', 'raise SystemExit(7)'], self.profile, self.evidence, lambda pid: True)
        self.assertFalse(self.profile.exists())

    def test_foreign_key_failure_is_rejected(self):
        self.profile.mkdir()
        with closing(sqlite3.connect(self.profile/'data.db')) as db, db:
            db.executescript('CREATE TABLE p(id PRIMARY KEY); CREATE TABLE c(id REFERENCES p(id)); INSERT INTO c VALUES (1);')
        with self.assertRaisesRegex(RuntimeError, "foreign-key"):
            SMOKE.checks.check_database(self.profile/'data.db')

    def test_cli_refuses_personal_desktops_before_opening_any_profile(self):
        env = dict(os.environ, GITHUB_ACTIONS='false')
        result = subprocess.run([sys.executable, str(SCRIPT), '--executable', str(self.app), '--evidence', str(self.evidence)], env=env, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('never a personal profile', result.stderr)
        self.assertFalse(self.profile.exists())

    @unittest.skipUnless(shutil.which('bash'), 'bash is needed to test the macOS wrapper refusal')
    def test_dmg_wrapper_refuses_personal_desktops_before_mounting(self):
        result = subprocess.run(['bash', str(SCRIPT.with_name('macos-dmg-smoke.sh')), 'missing.dmg'],
                                env=dict(os.environ, GITHUB_ACTIONS='false'), capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('never a personal desktop', result.stderr)


if __name__ == '__main__':
    unittest.main()
