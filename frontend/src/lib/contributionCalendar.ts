// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Pure, framework-free logic for the GitHub-style contribution calendar.
// Kept separate from the Svelte component so it can be unit-tested with
// `node --test` (see scripts/contribution-calendar.test.mjs).

export interface CalendarCell {
  date: string;
  value: number;
  level: number;
  isFuture: boolean;
}

export interface CalendarPoint {
  date: string;
  tests: number;
  time_ms: number;
  lessons: number;
}

export type CalendarMetric = 'tests' | 'time' | 'lessons';

/** Extract the numeric value for a metric from a point. */
export function metricValue(p: CalendarPoint, metric: CalendarMetric): number {
  switch (metric) {
    case 'time':
      return p.time_ms;
    case 'lessons':
      return p.lessons;
    case 'tests':
    default:
      return p.tests;
  }
}

/** GitHub-style intensity bucket (0..4) based on the max daily value. */
export function level(value: number, maxValue: number): number {
  if (value <= 0) return 0;
  if (maxValue <= 0) return 1;
  const ratio = value / maxValue;
  if (ratio <= 0.25) return 1;
  if (ratio <= 0.5) return 2;
  if (ratio <= 0.75) return 3;
  return 4;
}

/** Format a Date as a local YYYY-MM-DD (avoids UTC shift from toISOString). */
function toLocalISODate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

/**
 * Build the 365-day grid aligned to full weeks (Sunday start), ending today.
 * `today` is injected for deterministic tests.
 */
export function buildGrid(
  points: CalendarPoint[],
  today: Date,
  metric: CalendarMetric = 'tests',
): CalendarCell[] {
  const valuesByDate = new Map(
    points.map((p) => [p.date, metricValue(p, metric)]),
  );
  const maxValue = Math.max(...points.map((p) => metricValue(p, metric)), 0);

  const end = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  const start = new Date(end);
  start.setDate(end.getDate() - 364); // 365 days inclusive

  // Align start to Sunday so columns are full weeks.
  const startDow = start.getDay();
  start.setDate(start.getDate() - startDow);

  const cells: CalendarCell[] = [];
  const cursor = new Date(start);
  while (cursor <= end) {
    const iso = toLocalISODate(cursor);
    const value = valuesByDate.get(iso) ?? 0;
    cells.push({
      date: iso,
      value,
      level: level(value, maxValue),
      isFuture: cursor > end,
    });
    cursor.setDate(cursor.getDate() + 1);
  }
  return cells;
}

/** Human-readable tooltip for a cell. */
export function formatTooltip(
  c: { date: string; value: number },
  metric: CalendarMetric,
): string {
  const d = new Date(c.date + 'T00:00:00');
  const label = d.toLocaleDateString('en', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
  if (c.value <= 0) return `${label}: No activity`;
  if (metric === 'time') {
    const minutes = Math.round(c.value / 60000);
    return `${label}: ${minutes} min`;
  }
  const noun = metric === 'lessons' ? 'lesson' : 'test';
  return `${label}: ${c.value} ${noun}${c.value === 1 ? '' : 's'}`;
}
