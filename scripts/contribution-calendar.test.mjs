// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildGrid,
  formatTooltip,
  level,
  metricValue,
} from '../frontend/src/lib/contributionCalendar.ts';

test('level buckets map values to GitHub-style intensity 0..4', () => {
  // No activity -> 0
  assert.equal(level(0, 100), 0);
  // maxValue <= 0 with activity -> 1
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

test('metricValue extracts the right field per metric', () => {
  const p = { date: '2026-08-10', tests: 3, time_ms: 120000, lessons: 2 };
  assert.equal(metricValue(p, 'tests'), 3);
  assert.equal(metricValue(p, 'time'), 120000);
  assert.equal(metricValue(p, 'lessons'), 2);
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

test('buildGrid maps daily values onto the correct dates per metric', () => {
  const today = new Date('2026-08-10T12:00:00Z');
  const points = [{ date: '2026-08-10', tests: 5, time_ms: 300000, lessons: 1 }];

  const testsGrid = buildGrid(points, today, 'tests');
  assert.equal(testsGrid[testsGrid.length - 1].value, 5);
  assert.equal(testsGrid[testsGrid.length - 1].level, 4);

  const timeGrid = buildGrid(points, today, 'time');
  assert.equal(timeGrid[timeGrid.length - 1].value, 300000);

  const lessonsGrid = buildGrid(points, today, 'lessons');
  assert.equal(lessonsGrid[lessonsGrid.length - 1].value, 1);
});

test('buildGrid marks empty days as level 0 and none future', () => {
  const today = new Date('2026-08-10T12:00:00Z');
  const grid = buildGrid([], today);

  for (const cell of grid) {
    assert.equal(cell.level, 0);
    assert.equal(cell.isFuture, false);
  }
});

test('formatTooltip renders activity and no-activity labels per metric', () => {
  assert.equal(
    formatTooltip({ date: '2026-08-10', value: 0 }, 'tests'),
    'Aug 10, 2026: No activity',
  );
  assert.equal(
    formatTooltip({ date: '2026-08-10', value: 1 }, 'tests'),
    'Aug 10, 2026: 1 test',
  );
  assert.equal(
    formatTooltip({ date: '2026-08-10', value: 3 }, 'tests'),
    'Aug 10, 2026: 3 tests',
  );
  assert.equal(
    formatTooltip({ date: '2026-08-10', value: 2 }, 'lessons'),
    'Aug 10, 2026: 2 lessons',
  );
  assert.equal(
    formatTooltip({ date: '2026-08-10', value: 120000 }, 'time'),
    'Aug 10, 2026: 2 min',
  );
});
