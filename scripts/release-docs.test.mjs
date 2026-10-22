// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('README Vite major and current installation paths agree with shipped configuration', () => {
  const lock = JSON.parse(read('frontend/package-lock.json'));
  const viteMajor = lock.packages['node_modules/vite'].version.split('.')[0];
  assert.match(read('README.md'), new RegExp(`Vite ${viteMajor}\\b`));
  const { identifier } = JSON.parse(read('crates/app/tauri.conf.json'));
  const install = read('INSTALL.md');
  const rows = install.split('\n').filter(line => /^\| (SQLite database|Settings) /.test(line));
  assert.equal(rows.length, 2);
  for (const row of rows) assert.ok(row.includes(`/${identifier}/`));
  assert.doesNotMatch(install, /rm -rf/,'do not publish a legacy-only destructive uninstall recipe');
});
