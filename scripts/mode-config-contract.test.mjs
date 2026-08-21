// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Contract test for the typed mode_config shapes emitted by the backend.
// The Rust mode implementations (crates/core/src/modes/*) are the source of
// truth; this test pins the TypeScript ModeConfig union to those shapes so
// a backend change that breaks the frontend contract fails CI.

import test from 'node:test';
import assert from 'node:assert/strict';

import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function rustModeConfig(modeFile, modeName) {
  const source = readFileSync(
    join(repositoryRoot, 'crates', 'core', 'src', 'modes', modeFile),
    'utf8',
  );
  const match = source.match(/fn mode_config\(&self\) -> serde_json::Value \{([\s\S]*?)\n    \}/);
  assert.ok(match, `${modeName} mode_config body not found`);
  return match[1];
}

function jsonKeys(body) {
  const keys = [...body.matchAll(/"([a-z_]+)":/g)].map((match) => match[1]);
  return keys.sort();
}

test('time mode_config emits duration and language', () => {
  assert.deepEqual(jsonKeys(rustModeConfig('time.rs', 'time')), ['duration', 'language']);
});

test('words mode_config emits word_count and language', () => {
  assert.deepEqual(jsonKeys(rustModeConfig('words.rs', 'words')), ['language', 'word_count']);
});

test('quote mode_config emits quote_id and language', () => {
  assert.deepEqual(jsonKeys(rustModeConfig('quote.rs', 'quote')), ['language', 'quote_id']);
});

test('custom mode_config emits language only', () => {
  assert.deepEqual(jsonKeys(rustModeConfig('custom.rs', 'custom')), ['language']);
});

test('lesson mode_config emits lesson_id and module_id', () => {
  const source = readFileSync(
    join(repositoryRoot, 'crates', 'core', 'src', 'lesson.rs'),
    'utf8',
  );
  const match = source.match(/fn mode_config\(&self\) -> serde_json::Value \{([\s\S]*?)\n    \}/);
  assert.ok(match, 'lesson mode_config body not found');
  assert.deepEqual(jsonKeys(match[1]), ['lesson_id', 'module_id']);
});

test('the TypeScript ModeConfig union covers every backend shape', () => {
  const types = readFileSync(
    join(repositoryRoot, 'frontend', 'src', 'lib', 'types', 'index.ts'),
    'utf8',
  );
  const union = types.match(/export type ModeConfig =([\s\S]*?);\n\nexport interface TestSessionResponse/);
  assert.ok(union, 'ModeConfig union not found');
  const body = union[1];
  for (const [typeName, fields] of [
    ['time', ['duration', 'language']],
    ['words', ['word_count', 'language']],
    ['quote', ['quote_id', 'language']],
    ['custom', ['language']],
    ['lesson', ['lesson_id', 'module_id']],
  ]) {
    const variant = body.match(new RegExp(`\\{ type: '${typeName}';([^}]*)\\}`));
    assert.ok(variant, `ModeConfig missing the ${typeName} variant`);
    const variantFields = [...variant[1].matchAll(/([a-z_]+):/g)].map((match) => match[1]).sort();
    assert.deepEqual(variantFields, [...fields].sort(), `${typeName} variant fields mismatch`);
  }
});
