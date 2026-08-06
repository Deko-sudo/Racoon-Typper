// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import {
  MAX_PROFILE_FILE_BYTES,
  profileImportRows,
  validateProfileFileMetadata,
} from '../frontend/src/lib/profileTransfer.ts';

test('profile file metadata rejects empty, non-JSON, and oversized files', () => {
  assert.equal(validateProfileFileMetadata({ name: 'profile.json', size: 0 }), 'empty');
  assert.equal(validateProfileFileMetadata({ name: 'profile.db', size: 10 }), 'not_json');
  assert.equal(
    validateProfileFileMetadata({ name: 'profile.JSON', size: MAX_PROFILE_FILE_BYTES + 1 }),
    'too_large',
  );
});

test('profile file metadata accepts a bounded JSON document', () => {
  assert.equal(validateProfileFileMetadata({ name: 'backup.JSON', size: 128 }), null);
  assert.equal(
    validateProfileFileMetadata({ name: 'maximum.json', size: MAX_PROFILE_FILE_BYTES }),
    null,
  );
});

test('profile import rows preserve every collection and count', () => {
  const rows = profileImportRows({
    policy: 'replace',
    tests: { incoming: 4, existing: 2, to_insert: 4 },
    personal_bests: { incoming: 1, existing: 1, to_insert: 1 },
    daily_stats: { incoming: 3, existing: 2, to_insert: 3 },
    streaks: { incoming: 2, existing: 1, to_insert: 2 },
    custom_texts: { incoming: 5, existing: 3, to_insert: 5 },
    lesson_progress: { incoming: 6, existing: 4, to_insert: 6 },
  });

  assert.deepEqual(rows.map((row) => row.key), [
    'tests',
    'personal_bests',
    'daily_stats',
    'streaks',
    'custom_texts',
    'lesson_progress',
  ]);
  assert.deepEqual(rows.at(-1), {
    key: 'lesson_progress',
    incoming: 6,
    existing: 4,
    toInsert: 6,
  });
});
