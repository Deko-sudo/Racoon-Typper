# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("smoke", ROOT / "scripts/linux-runtime-smoke.py")
SMOKE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SMOKE)
FAKE_APP = '''#!/usr/bin/env python3
import os,sqlite3,time
from pathlib import Path
p=Path(os.environ['XDG_DATA_HOME'])/'com.racoon.typper/data.db';p.parent.mkdir(parents=True,exist_ok=True)
with open(os.environ['SMOKE_AUDIT'],'a') as log: log.write(('reused' if p.exists() else 'new')+'\\n')
if os.environ.get('SMOKE_LOSE_DATA')=='1': p.unlink(missing_ok=True)
with sqlite3.connect(p) as db:
 db.execute('CREATE TABLE IF NOT EXISTS refinery_schema_history(version INTEGER)')
 if not db.execute('SELECT count(*) FROM refinery_schema_history').fetchone()[0]: db.executemany('INSERT INTO refinery_schema_history VALUES (?)',[(n,) for n in range(1,10)])
 db.execute('CREATE TABLE IF NOT EXISTS custom_texts(name TEXT,text TEXT,language TEXT,created_at TEXT,use_count INTEGER)')
while True: time.sleep(.1)
'''


class LinuxSmokeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name)
        self.app = self.path / "candidate.AppImage"
        self.app.write_text(FAKE_APP)
        self.app.chmod(0o700)
        self.old = dict(os.environ)
        self.addCleanup(lambda: (os.environ.clear(), os.environ.update(self.old)))
        os.environ['SMOKE_AUDIT'] = str(self.path/'audit')

    def test_preserves_marker_and_parent_environment(self):
        result = SMOKE.run_smoke([str(self.app)], self.path/'profile', timeout=3)
        self.assertTrue(result['persistent_custom_text'])
        self.assertEqual((self.path/'audit').read_text().splitlines(), ['new','reused'])
        self.assertEqual(os.environ.get('XDG_DATA_HOME'), self.old.get('XDG_DATA_HOME'))

    def test_rejects_a_process_that_recreates_the_database(self):
        os.environ['SMOKE_LOSE_DATA'] = '1'
        with self.assertRaisesRegex(RuntimeError, 'lost or altered'):
            SMOKE.run_smoke([str(self.app)], self.path/'profile', timeout=3)

    def test_rejects_early_exit_and_an_existing_profile(self):
        with self.assertRaisesRegex(RuntimeError, 'exited'):
            SMOKE.run_smoke(['/bin/false'], self.path/'profile', timeout=1)
        profile = self.path/'existing';(profile/'data/com.racoon.typper').mkdir(parents=True)
        (profile/'data/com.racoon.typper/data.db').touch()
        with self.assertRaisesRegex(RuntimeError, 'NEW disposable'):
            SMOKE.run_smoke([str(self.app)], profile, timeout=1)

    def test_appimage_wrapper_reuses_profile_and_never_calls_broad_pkill(self):
        fakebin=self.path/'bin';fakebin.mkdir()
        for name,content in {
            'sudo':'#!/bin/sh\nexit 0\n',
            'dbus-run-session':'#!/bin/sh\n[ "$1" != -- ] || shift\nexec "$@"\n',
            'xvfb-run':'#!/bin/sh\n[ "$1" != -a ] || shift\nexec "$@"\n',
            'pkill':'#!/bin/sh\necho pkill >> "$SMOKE_AUDIT"\nexit 0\n',
        }.items():
            p=fakebin/name;p.write_text(content);p.chmod(0o700)
        env=dict(os.environ,PATH=str(fakebin)+':'+os.environ['PATH'],APPIMAGE_SMOKE_MODE='extract')
        script=Path(os.environ.get('SMOKE_APPIMAGE_SCRIPT',ROOT/'scripts/appimage-smoke.sh'))
        result=subprocess.run(['bash',str(script),str(self.app)],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=20)
        self.assertEqual(result.returncode,0,result.stdout)
        self.assertEqual((self.path/'audit').read_text().splitlines(),['new','reused'],result.stdout)


if __name__ == '__main__': unittest.main()
