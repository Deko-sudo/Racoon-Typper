// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

import {
  formatForFile,
  summarizePlan,
  TEXT_PACK_PLAN_FIELDS,
} from '../frontend/src/lib/textPack.ts';

test('file extension selects the explicit source format', () => {
  assert.equal(formatForFile('pack.json'), 'json');
  assert.equal(formatForFile('anki-deck.CSV'), 'csv');
  assert.equal(formatForFile('deck.tsv'), 'tsv');
  assert.equal(formatForFile('notes.txt'), 'blocks');
  assert.equal(formatForFile('no-extension'), 'blocks');
});

test('plan summary renders insert/skip/remove counters', () => {
  const plan = {
    policy: 'replace',
    source_format: 'csv',
    language: 'en',
    incoming: 5,
    duplicates_in_pack: 1,
    existing_in_language: 3,
    to_insert: 4,
    to_skip: 0,
    removed_by_replace: 3,
  };
  assert.equal(summarizePlan(plan), '+4 / ~0 / −3');
});

test('wire plan fields stay in lockstep with the Rust struct', () => {
  // The TS interface and serde struct must not drift; parse the Rust source
  // for its public field list exactly like mode-config-contract does.
  const rust = readFileSync(join(repositoryRoot, 'crates/data/src/text_pack.rs'), 'utf8');
  const structMatch = rust.match(
    /pub struct TextPackImportPlan \{([\s\S]*?)\n\}/,
  );
  assert.ok(structMatch, 'TextPackImportPlan struct not found');
  const rustFields = [...structMatch[1].matchAll(/pub (\w+):/g)].map((m) => m[1]);
  assert.deepEqual(rustFields, [...TEXT_PACK_PLAN_FIELDS]);
});
