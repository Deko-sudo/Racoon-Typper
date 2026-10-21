// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import { findMatches, vimActionForKey } from '../frontend/src/lib/vimNavigation.ts';

test('h/l navigate between tabs, bounded at the edges', () => {
  // From 'test' (index 1), h goes to dashboard, l goes to pomodoro.
  assert.deepEqual(vimActionForKey('h', 'test', false), {
    action: { type: 'prev_tab' },
    nextPendingG: false,
  });
  assert.deepEqual(vimActionForKey('l', 'test', false), {
    action: { type: 'next_tab' },
    nextPendingG: false,
  });
  // From 'pomodoro' (index 2), h goes back to test, l goes to lessons.
  assert.deepEqual(vimActionForKey('h', 'pomodoro', false), {
    action: { type: 'prev_tab' },
    nextPendingG: false,
  });
  assert.deepEqual(vimActionForKey('l', 'pomodoro', false), {
    action: { type: 'next_tab' },
    nextPendingG: false,
  });
  // At the first tab, h does nothing.
  assert.deepEqual(vimActionForKey('h', 'dashboard', false), {
    action: { type: 'none' },
    nextPendingG: false,
  });
  // At the last tab, l does nothing.
  assert.deepEqual(vimActionForKey('l', 'settings', false), {
    action: { type: 'none' },
    nextPendingG: false,
  });
});

test('j/k scroll, gg scrolls to top, G scrolls to bottom', () => {
  assert.deepEqual(vimActionForKey('j', 'test', false), {
    action: { type: 'scroll_down' },
    nextPendingG: false,
  });
  assert.deepEqual(vimActionForKey('k', 'test', false), {
    action: { type: 'scroll_up' },
    nextPendingG: false,
  });
  // Single g sets pendingG; second g scrolls to top.
  assert.deepEqual(vimActionForKey('g', 'test', false), {
    action: { type: 'none' },
    nextPendingG: true,
  });
  assert.deepEqual(vimActionForKey('g', 'test', true), {
    action: { type: 'scroll_top' },
    nextPendingG: false,
  });
  assert.deepEqual(vimActionForKey('G', 'test', false), {
    action: { type: 'scroll_bottom' },
    nextPendingG: false,
  });
});

test('r restarts the test', () => {
  assert.deepEqual(vimActionForKey('r', 'test', false), {
    action: { type: 'restart' },
    nextPendingG: false,
  });
});

test('unknown keys produce no action', () => {
  assert.deepEqual(vimActionForKey('x', 'test', false), {
    action: { type: 'none' },
    nextPendingG: false,
  });
});

test('findMatches highlights every case-insensitive occurrence', () => {
  assert.deepEqual([...findMatches('hello world hello', 'hello')].sort((a, b) => a - b), [0, 1, 2, 3, 4, 12, 13, 14, 15, 16]);
  assert.deepEqual([...findMatches('Hello World', 'hello')].sort((a, b) => a - b), [0, 1, 2, 3, 4]);
  assert.deepEqual([...findMatches('abc', 'z')], []);
});

test('findMatches ignores empty and whitespace-only queries', () => {
  assert.equal(findMatches('abc', '').size, 0);
  assert.equal(findMatches('abc', '   ').size, 0);
});
