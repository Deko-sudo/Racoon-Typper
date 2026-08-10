// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Pure, framework-free logic for the GitHub-style contribution calendar.
// Kept separate from the Svelte component so it can be unit-tested with
// `node --test` (see scripts/contribution-calendar.test.mjs).

export interface CalendarCell {
  date: string;
  tests: number;
  level: number;
  isFuture: boolean;
}

export interface CalendarPoint {
  date: string;
  tests: number;
}

/** GitHub-style intensity bucket (0..4) based on the max daily test count. */
export function level(tests: number, maxTests: number): number {
  if (tests <= 0) return 0;
  if (maxTests <= 0) return 1;
  const ratio = tests / maxTests;
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
): CalendarCell[] {
  const testsByDate = new Map(points.map((p) => [p.date, p.tests]));
  const maxTests = Math.max(...points.map((p) => p.tests), 0);

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
    const tests = testsByDate.get(iso) ?? 0;
    cells.push({
      date: iso,
      tests,
      level: level(tests, maxTests),
      isFuture: cursor > end,
    });
    cursor.setDate(cursor.getDate() + 1);
  }
  return cells;
}

/** Human-readable tooltip for a cell. */
export function formatTooltip(c: { date: string; tests: number }): string {
  const d = new Date(c.date + 'T00:00:00');
  const label = d.toLocaleDateString('en', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
  if (c.tests <= 0) return `${label}: No activity`;
  return `${label}: ${c.tests} test${c.tests === 1 ? '' : 's'}`;
}
