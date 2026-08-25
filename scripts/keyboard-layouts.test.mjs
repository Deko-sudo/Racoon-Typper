// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import {
  DVORAK_FINGERS,
  DVORAK_ROWS,
  layoutFingers,
  layoutRows,
  ROWS,
  RU_FINGERS,
  RU_ROWS,
} from '../frontend/src/lib/keyboard.ts';

const FINGER_CODES = new Set(['LP', 'LR', 'LM', 'LI', 'RI', 'RM', 'RR', 'RP']);

function allKeys(rows) {
  return rows.flat();
}

test('dvorak tables cover the same physical positions as qwerty', () => {
  assert.equal(DVORAK_ROWS.length, ROWS.length);
  DVORAK_ROWS.forEach((row, index) => {
    assert.equal(row.length, ROWS[index].length, `row ${index} length mismatch`);
  });
  // Physical home-row anchors keep their finger roles across layouts.
  assert.equal(DVORAK_FINGERS.a, 'LP');
  assert.equal(DVORAK_FINGERS.e, 'LM');
  assert.equal(DVORAK_FINGERS.i, 'LI');
  assert.equal(DVORAK_FINGERS.d, 'RI');
  assert.equal(DVORAK_FINGERS.t, 'RM');
  assert.equal(DVORAK_FINGERS.n, 'RR');
  assert.equal(DVORAK_FINGERS.s, 'RP');
});

test('every dvorak key has a valid finger assignment', () => {
  for (const key of allKeys(DVORAK_ROWS)) {
    const finger = DVORAK_FINGERS[key];
    assert.ok(finger, `missing finger for dvorak key ${key}`);
    assert.ok(FINGER_CODES.has(finger), `invalid finger code ${finger} for ${key}`);
  }
});

test('layout helpers dispatch by cyrillic first, then by selected layout', () => {
  // Cyrillic always renders JCUKEN regardless of the setting.
  assert.deepEqual(layoutRows('dvorak', true), RU_ROWS);
  assert.deepEqual(layoutFingers('dvorak', true), RU_FINGERS);

  assert.deepEqual(layoutRows('qwerty', false), ROWS);
  assert.deepEqual(layoutRows('dvorak', false), DVORAK_ROWS);

  // Unknown values fall back to qwerty instead of crashing.
  assert.deepEqual(layoutRows('colemak', false), ROWS);
  assert.deepEqual(layoutFingers(undefined, false), layoutFingers('qwerty', false));
});

test('russian tables remain intact through the helpers', () => {
  assert.deepEqual(layoutRows('jcuken', true), RU_ROWS);
  for (const key of allKeys(RU_ROWS)) {
    assert.ok(RU_FINGERS[key], `missing russian finger for ${key}`);
  }
});
