// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readdir, mkdir, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Readable } from 'node:stream';
import { AUDIT_ORIGIN, authorized, browserAuditPlugin, readBody, startBackend } from './browser-audit-server.mjs';

test('audit mode refuses production builds and non-loopback server overrides', () => {
  const plugin = browserAuditPlugin();
  assert.throws(() => plugin.configResolved({ command: 'build' }), /development-only/);
  assert.throws(() => plugin.configResolved({ command: 'serve', server: { host: '0.0.0.0', port: 1440, strictPort: true } }), /loopback/);
  assert.doesNotThrow(() => plugin.configResolved({ command: 'serve', server: { host: '127.0.0.1', port: 1440, strictPort: true } }));
});

test('audit transport requires exact local host, origin and per-launch token', () => {
  const headers = { host: '127.0.0.1:1440', origin: AUDIT_ORIGIN, authorization: 'Bearer secret' };
  assert.equal(authorized(headers, 'secret'), true);
  for (const replacement of [
    { host: 'evil.example:1440' }, { origin: 'https://evil.example' },
    { origin: undefined }, { authorization: undefined }, { authorization: 'Bearer other' },
  ]) assert.equal(authorized({ ...headers, ...replacement }, 'secret'), false);
});

test('audit body limits are enforced while streaming, including missing content-length', async () => {
  assert.equal((await readBody(Readable.from([Buffer.from('abc')]), 3)).toString(), 'abc');
  await assert.rejects(readBody(Readable.from([Buffer.from('ab'), Buffer.from('cd')]), 3), /too large/);
});

test('a binary without the audit feature refuses the flag before native profile setup', {
  skip: !process.env.RACOON_NATIVE_BINARY && 'Set RACOON_NATIVE_BINARY to a build without browser-audit.',
}, async () => {
  const profile = await mkdtemp(join(tmpdir(), 'racoon-browser-disabled-'));
  const result = spawnSync(process.env.RACOON_NATIVE_BINARY, ['--browser-audit'], {
    env: { ...process.env, XDG_DATA_HOME: profile, XDG_CONFIG_HOME: profile }, encoding: 'utf8', timeout: 5000,
  });
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /Browser audit requires a debug build/);
  assert.deepEqual(await readdir(profile), []);
});

test('real command bridge migrates a private database and persists a completed test', {
  skip: !process.env.RACOON_BROWSER_BINARY && 'Build browser-audit and set RACOON_BROWSER_BINARY.',
}, async () => {
  const profile = await mkdtemp(join(tmpdir(), 'racoon-browser-contract-'));
  const backend = await startBackend(process.env.RACOON_BROWSER_BINARY, profile);
  try {
    const call = async (cmd, args = {}) => {
      const result = await backend.invoke({ cmd, args });
      assert.equal(result.ok, true, JSON.stringify(result));
      return result.value;
    };
    const [settings, themes] = await Promise.all([call('get_settings'), call('get_themes')]);
    assert.equal(settings.practice_language, 'en');
    assert.equal(themes.length, 50);
    await call('set_setting', { key: 'ui_language', value: 'ru' });
    assert.equal((await call('get_settings')).ui_language, 'ru');
    const id = await call('save_custom_text', { name: 'Browser bridge contract', text: 'фыва', language: 'ru' });
    const session = await call('start_custom_text_test', { customTextId: id });
    let output;
    for (const key of 'фыва') output = await call('process_key', { key, code: 'KeyA', sessionId: session.session_id });
    assert.equal(output.session_state, 'persisted');
    assert.equal((await call('get_stats_history', { limit: 10, offset: 0 })).total, 1);
    await call('clear_statistics');
    assert.equal((await call('get_stats_history', { limit: 10, offset: 0 })).total, 0);
    assert.equal((await call('get_custom_texts', { limit: 10 })).length, 1);
    for (const request of [
      { cmd: 'restore_database', args: { backupPath: '/etc/passwd' } },
      { cmd: 'plugin:fs|read_text_file', args: { path: '/etc/passwd' } },
      { cmd: 'unknown_command', args: {} },
    ]) assert.equal((await backend.invoke(request)).ok, false);
  } finally {
    backend.close();
  }
});

test('real restore command refuses a running session before validating selected SQLite', {
  skip: !process.env.RACOON_BROWSER_BINARY && 'Build browser-audit and set RACOON_BROWSER_BINARY.',
}, async () => {
  const profile = await mkdtemp(join(tmpdir(), 'racoon-browser-restore-active-'));
  const backend = await startBackend(process.env.RACOON_BROWSER_BINARY, profile);
  try {
    const invoke = (cmd, args = {}) => backend.invoke({cmd, args});
    const saved = await invoke('save_custom_text', {name:'Running restore guard', text:'фыва', language:'ru'});
    assert.equal(saved.ok, true);
    const session = await invoke('start_custom_text_test', {customTextId:saved.value});
    assert.equal(session.ok, true);
    const input = await invoke('process_key', {key:'ф',code:'KeyA',sessionId:session.value.session_id});
    assert.equal(input.ok, true);
    const uploads=join(profile,'uploads');await mkdir(uploads,{mode:0o700});
    const selected=join(uploads,'not-a-backup.db');await writeFile(selected,'not SQLite',{mode:0o600});
    const restore = await invoke('restore_database', {backupPath:selected});
    assert.equal(restore.ok, false);
    assert.equal(restore.error.code, 'TEST_ALREADY_ACTIVE');
    assert.doesNotMatch(JSON.stringify(restore), /not-a-backup/);
    const history = await invoke('get_stats_history', {limit:10,offset:0});
    assert.equal(history.ok, true);assert.equal(history.value.total, 0);
  } finally {backend.close();}
});
