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
  isToday: boolean;
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

/**
 * Numeric legend ranges for the five intensity buckets, derived from the
 * maximum daily value of the current metric. Returns [min, max] pairs
 * (inclusive); the zero bucket is always [0, 0]. Buckets with no integer
 * value (possible when maxValue < 4) are `null` and render as "—".
 */
export function legendRanges(maxValue: number): Array<[number, number] | null> {
  if (maxValue <= 0) return [[0, 0], null, null, null, null];
  const ranges: Array<[number, number] | null> = [[0, 0]];
  for (let bucket = 1; bucket <= 4; bucket++) {
    const lo = Math.floor(((bucket - 1) * maxValue) / 4) + 1;
    const hi = Math.floor((bucket * maxValue) / 4);
    ranges.push(lo <= hi ? [lo, hi] : null);
  }
  return ranges;
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
  const todayIso = toLocalISODate(end);
  while (cursor <= end) {
    const iso = toLocalISODate(cursor);
    const value = valuesByDate.get(iso) ?? 0;
    cells.push({
      date: iso,
      value,
      level: level(value, maxValue),
      isFuture: cursor > end,
      isToday: iso === todayIso,
    });
    cursor.setDate(cursor.getDate() + 1);
  }
  return cells;
}

/** Plural category for a count in the given locale ('one'|'few'|'many'|'other'). */
function pluralForm(lang: string, n: number): string {
  try {
    return new Intl.PluralRules(lang).select(n);
  } catch {
    return n === 1 ? 'one' : 'other';
  }
}

export interface TooltipLabels {
  noActivity: string;
  minutes: string;
  lesson: Record<string, string>;
  test: Record<string, string>;
}

/** Human-readable tooltip for a cell, localized to the UI language. */
export function formatTooltip(
  c: { date: string; value: number },
  metric: CalendarMetric,
  lang = 'en',
  labels?: TooltipLabels,
): string {
  const d = new Date(c.date + 'T00:00:00');
  const label = d.toLocaleDateString(lang, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
  const noActivity = labels?.noActivity ?? 'No activity';
  if (c.value <= 0) return `${label}: ${noActivity}`;
  if (metric === 'time') {
    const minutes = Math.round(c.value / 60000);
    return `${label}: ${minutes} ${labels?.minutes ?? 'min'}`;
  }
  const form = pluralForm(lang, c.value);
  if (metric === 'lessons') {
    const noun = labels?.lesson?.[form] ?? (form === 'one' ? 'lesson' : 'lessons');
    return `${label}: ${c.value} ${noun}`;
  }
  const noun = labels?.test?.[form] ?? (form === 'one' ? 'test' : 'tests');
  return `${label}: ${c.value} ${noun}`;
}
