// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import {
  DVORAK_FINGERS,
  DVORAK_ROWS,
  fingerForKey,
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

test('hand guide: required physical-key -> finger pairs (EN/RU)', () => {
  const cases = [
    // [latin, cyrillic, expected finger]
    ['a', 'ф', 'LP'],
    ['s', 'ы', 'LR'],
    ['d', 'в', 'LM'],
    ['f', 'а', 'LI'],
    ['j', 'о', 'RI'],
    ['k', 'л', 'RM'],
    ['l', 'д', 'RR'],
    [';', 'ж', 'RP'],
  ];
  for (const [latin, cyrillic, expected] of cases) {
    assert.equal(
      fingerForKey(latin, 'qwerty', false), expected,
      `latin '${latin}' must activate ${expected}`,
    );
    assert.equal(
      fingerForKey(cyrillic, 'qwerty', true), expected,
      `cyrillic '${cyrillic}' must activate ${expected} (physical position)`,
    );
    // The layout tables used for key coloring must agree.
    assert.equal(layoutFingers('qwerty', false)[latin], expected);
    assert.equal(layoutFingers('qwerty', true)[cyrillic], expected);
  }
});

test('number row and punctuation keep sensible touch-typing fingers', () => {
  assert.equal(fingerForKey('1', 'qwerty', false), 'LP');
  assert.equal(fingerForKey('5', 'qwerty', false), 'LI');
  assert.equal(fingerForKey('6', 'qwerty', false), 'RI');
  assert.equal(fingerForKey('0', 'qwerty', false), 'RP');
  assert.equal(fingerForKey('-', 'qwerty', false), 'RP');
  assert.equal(fingerForKey('[', 'qwerty', false), 'RP');
  assert.equal(fingerForKey("'", 'qwerty', false), 'RP');
  // Russian digits live on the same physical keys.
  assert.equal(fingerForKey('3', 'jcuken', true), 'LM');
});

test('space uses the right thumb; unknown keys highlight nothing', () => {
  assert.equal(fingerForKey(' ', 'qwerty', false), 'RT');
  assert.equal(fingerForKey('', 'qwerty', false), '');
  assert.equal(fingerForKey('€', 'qwerty', false), '');
  assert.equal(fingerForKey('Ω', 'jcuken', true), '');
});

test('layout switching preserves physical fingers', () => {
  // Same physical key under Dvorak: 'a' home-left pinky, 's' home-right pinky.
  assert.equal(fingerForKey('a', 'dvorak', false), 'LP');
  assert.equal(fingerForKey('s', 'dvorak', false), 'RP');
  assert.equal(fingerForKey('e', 'dvorak', false), 'LM');
  // Cyrillic input ignores the Latin layout selection entirely.
  assert.equal(fingerForKey('л', 'dvorak', true), 'RM');
  assert.equal(fingerForKey('л', 'qwerty', true), 'RM');
});

test('every positioned key across all layouts resolves to a finger', () => {
  for (const [name, rows] of [['qwerty', ROWS], ['jcuken', RU_ROWS], ['dvorak', DVORAK_ROWS]]) {
    for (const row of rows) {
      for (const key of row) {
        assert.match(layoutFingers(name === 'jcuken' ? 'jcuken' : name, name === 'jcuken')[key] ?? '', /^(LP|LR|LM|LI|RI|RM|RR|RP)$/, `${name}: missing finger for '${key}'`);
      }
    }
  }
});
