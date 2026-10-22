#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, cpSync, mkdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('../', import.meta.url));
const fixture = resolve(root, 'scripts/macro-diagnostics-regression');
const baseline = process.argv.includes('--baseline');
const directory = mkdtempSync(join(tmpdir(), 'racoon-macro-contract-'));
const logs = process.env.RACOON_MACRO_LOG_DIR;
if (logs) mkdirSync(logs, { recursive: true });
const failures = [];
const results = [];
function run(name, args) {
  const result = spawnSync('cargo', [...args, '--manifest-path', join(directory, 'Cargo.toml')], {
    encoding: 'utf8', timeout: 240_000, maxBuffer: 64 * 1024 * 1024,
  });
  if (logs) writeFileSync(join(logs, `${name}.log`), `${result.stdout ?? ''}${result.stderr ?? ''}`);
  if (result.error) throw result.error;
  return result;
}
function record(name, check) {
  try { check(); results.push({ name, passed: true }); }
  catch (error) { failures.push(name); results.push({ name, passed: false, error: error.message }); }
}

try {
  cpSync(join(fixture, 'src'), join(directory, 'src'), { recursive: true });
  cpSync(join(fixture, 'cases'), join(directory, 'cases'), { recursive: true });
  cpSync(join(fixture, 'Cargo.lock'), join(directory, 'Cargo.lock'));
  let manifest = readFileSync(join(fixture, 'Cargo.toml'), 'utf8')
    .replaceAll('../../vendor/', `${resolve(root, 'vendor').replaceAll('\\', '/')}/`);
  if (baseline) manifest = manifest.replace(/^(?:glib-macros|gtk3-macros) = \{ path = .*\}\n/gm, '');
  const cases = JSON.parse(readFileSync(join(fixture, 'cases/cases.json')));
  for (const item of cases) manifest += `\n[[bin]]\nname = "${item.name}"\npath = "cases/${item.name}.rs"\n`;
  writeFileSync(join(directory, 'Cargo.toml'), manifest);
  // The baseline is reference-only: resolve the two original registry macros
  // against the same pinned fixture lock. Candidate CI always uses --locked.
  const metadata = run('metadata', ['metadata', '--offline', ...(baseline ? [] : ['--locked']), '--format-version', '1']);
  assert.equal(metadata.status, 0, metadata.stderr);
  const packages = JSON.parse(metadata.stdout).packages;
  record('old diagnostic helper absent from resolved graph', () => {
    assert.ok(!packages.some(pkg => ['proc-macro-error', 'proc-macro-error-attr'].includes(pkg.name)),
      'The old diagnostic helper is still resolved');
  });
  const valid = run('valid', ['test', '--offline', '--locked', '--lib']);
  record('valid macro expansion and runtime roundtrips', () => assert.equal(valid.status, 0, valid.stderr));
  for (const item of cases) {
    const result = run(item.name, ['check', '--offline', '--locked', '--bin', item.name, '--message-format=json']);
    record(item.name, () => {
      assert.equal(result.status, 101, 'Invalid input must fail compilation');
      const messages = result.stdout.split('\n').filter(line => line.startsWith('{')).map(line => JSON.parse(line))
        .filter(event => event.reason === 'compiler-message').map(event => event.message);
      assert.ok(!messages.some(message => /proc.?macro.*panicked/.test(message.message)), 'Macro must report an error, not panic');
      const diagnostic = messages.find(message => message.level === 'error' && message.message.includes(item.message));
      assert.ok(diagnostic, `Missing diagnostic: ${item.message}`);
      assert.ok(diagnostic.spans.some(span => span.is_primary && span.file_name.endsWith(`${item.name}.rs`)
        && span.line_start === item.line), `Expected a primary input span on line ${item.line}`);
    });
  }
} finally {
  rmSync(directory, { recursive: true, force: true });
  if (logs) writeFileSync(join(logs, 'results.json'), JSON.stringify({ baseline, results }, null, 2) + '\n');
}
for (const result of results) console.log(`${result.passed ? 'PASS' : 'FAIL'} ${result.name}${result.error ? `: ${result.error}` : ''}`);
console.log(`${results.length - failures.length}/${results.length} macro contracts passed (${baseline ? 'registry baseline' : 'candidate'})`);
if (failures.length) process.exitCode = 1;
