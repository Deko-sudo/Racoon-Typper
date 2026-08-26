// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Pure Vim-navigation logic, extracted from App.svelte so it can be unit-tested
// and reused. The component wires this to the DOM (scroll, view switching).

export type VimAction =
  | { type: 'prev_tab' }
  | { type: 'next_tab' }
  | { type: 'scroll_up' }
  | { type: 'scroll_down' }
  | { type: 'scroll_top' }
  | { type: 'scroll_bottom' }
  | { type: 'restart' }
  | { type: 'none' };

export const VIM_VIEWS = [
  'dashboard',
  'test',
  'pomodoro',
  'lessons',
  'weakkeys',
  'analytics',
  'achievements',
  'history',
  'bests',
  'custom',
  'settings',
] as const;

export type VimView = (typeof VIM_VIEWS)[number];

/**
 * Case-insensitive substring search over the test text. Returns the set of
 * character positions covered by every match (visual highlight only — the
 * backend caret is never moved by search).
 */
export function findMatches(text: string, query: string): Set<number> {
  const matches = new Set<number>();
  const needle = query.trim().toLowerCase();
  if (!needle) return matches;
  const haystack = text.toLowerCase();
  let index = haystack.indexOf(needle);
  while (index !== -1) {
    for (let i = index; i < index + needle.length; i++) matches.add(i);
    index = haystack.indexOf(needle, index + 1);
  }
  return matches;
}

/**
 * Map a keypress to a Vim action given the current view index.
 * `pendingG` tracks a single 'g' press so 'gg' becomes scroll_top.
 * Returns the action and the next pendingG state.
 */
export function vimActionForKey(
  key: string,
  currentView: string,
  pendingG: boolean,
): { action: VimAction; nextPendingG: boolean } {
  const currentIdx = VIM_VIEWS.indexOf(currentView as VimView);

  switch (key) {
    case 'h':
      return {
        action: currentIdx > 0 ? { type: 'prev_tab' } : { type: 'none' },
        nextPendingG: false,
      };
    case 'l':
      return {
        action: currentIdx >= 0 && currentIdx < VIM_VIEWS.length - 1
          ? { type: 'next_tab' }
          : { type: 'none' },
        nextPendingG: false,
      };
    case 'k':
      return { action: { type: 'scroll_up' }, nextPendingG: false };
    case 'j':
      return { action: { type: 'scroll_down' }, nextPendingG: false };
    case 'g':
      if (pendingG) return { action: { type: 'scroll_top' }, nextPendingG: false };
      return { action: { type: 'none' }, nextPendingG: true };
    case 'G':
      return { action: { type: 'scroll_bottom' }, nextPendingG: false };
    case 'r':
      return { action: { type: 'restart' }, nextPendingG: false };
    default:
      return { action: { type: 'none' }, nextPendingG: false };
  }
}
