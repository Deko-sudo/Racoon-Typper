// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildGrid,
  formatTooltip,
  level,
} from '../frontend/src/lib/contributionCalendar.ts';

test('level buckets map test counts to GitHub-style intensity 0..4', () => {
  // No activity -> 0
  assert.equal(level(0, 100), 0);
  // maxTests <= 0 with activity -> 1
  assert.equal(level(5, 0), 1);
  // ratio <= 0.25 -> 1
  assert.equal(level(10, 100), 1);
  // ratio <= 0.5 -> 2
  assert.equal(level(30, 100), 2);
  // ratio <= 0.75 -> 3
  assert.equal(level(60, 100), 3);
  // ratio > 0.75 -> 4
  assert.equal(level(80, 100), 4);
  assert.equal(level(100, 100), 4);
});

test('buildGrid produces a full-week-aligned 365-day grid ending today', () => {
  const today = new Date('2026-08-10T12:00:00Z');
  const grid = buildGrid([], today);

  // The grid is aligned to Sunday, so it may include a few days before the
  // 365-day window. It must always end on today.
  assert.ok(grid.length >= 365);
  assert.equal(grid[grid.length - 1].date, '2026-08-10');

  // First cell is a Sunday.
  const first = new Date(grid[0].date + 'T00:00:00');
  assert.equal(first.getDay(), 0);

  // Every cell is a valid date string.
  for (const cell of grid) {
    assert.match(cell.date, /^\d{4}-\d{2}-\d{2}$/);
  }
});

test('buildGrid maps daily test counts onto the correct dates', () => {
  const today = new Date('2026-08-10T12:00:00Z');
  const grid = buildGrid(
    [{ date: '2026-08-10', tests: 5 }],
    today,
  );

  const last = grid[grid.length - 1];
  assert.equal(last.tests, 5);
  assert.equal(last.level, 4); // only day with activity -> max -> level 4
});

test('buildGrid marks future days as future and empty days as level 0', () => {
  const today = new Date('2026-08-10T12:00:00Z');
  const grid = buildGrid([], today);

  // No activity anywhere -> all level 0, none future (grid ends today).
  for (const cell of grid) {
    assert.equal(cell.level, 0);
    assert.equal(cell.isFuture, false);
  }
});

test('formatTooltip renders activity and no-activity labels', () => {
  assert.equal(
    formatTooltip({ date: '2026-08-10', tests: 0 }),
    'Aug 10, 2026: No activity',
  );
  assert.equal(
    formatTooltip({ date: '2026-08-10', tests: 1 }),
    'Aug 10, 2026: 1 test',
  );
  assert.equal(
    formatTooltip({ date: '2026-08-10', tests: 3 }),
    'Aug 10, 2026: 3 tests',
  );
});
