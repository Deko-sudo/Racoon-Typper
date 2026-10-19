// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Presentation of backend first-attempt counts, keyed by expected Unicode scalar.
// This is deliberately not a reconstruction of a physical keyboard layout.
export interface HeatmapCounts {
  total_attempts: number;
  correct: number;
  incorrect: number;
}
export interface CharacterTile extends HeatmapCounts {
  symbol: string;
  codePoint: string;
}
export interface CharacterHeatmap {
  tiles: CharacterTile[];
  symbolCount: number;
  totals: HeatmapCounts;
  remaining: HeatmapCounts & { symbolCount: number };
}
export const MAX_HEATMAP_TILES = 60;

export function buildCharacterHeatmap(heatmap: Record<string, HeatmapCounts>): CharacterHeatmap {
  const totals: HeatmapCounts = { total_attempts: 0, correct: 0, incorrect: 0 };
  const tiles: CharacterTile[] = [];
  for (const [symbol, counts] of Object.entries(heatmap)) {
    const code = symbol.codePointAt(0);
    if (Array.from(symbol).length !== 1 || code === undefined || (code >= 0xd800 && code <= 0xdfff)
      || !counts || !['total_attempts', 'correct', 'incorrect'].every(field => {
        const value = counts[field as keyof HeatmapCounts];
        return Number.isSafeInteger(value) && value >= 0;
      }) || counts.correct + counts.incorrect !== counts.total_attempts) {
      throw new Error('Invalid character heatmap data');
    }
    if (!counts.total_attempts) continue;
    for (const field of ['total_attempts', 'correct', 'incorrect'] as const) {
      totals[field] += counts[field];
      if (!Number.isSafeInteger(totals[field])) throw new Error('Character heatmap counts exceed the supported range');
    }
    tiles.push({ symbol, codePoint: `U+${code.toString(16).toUpperCase().padStart(4, '0')}`,
      total_attempts: counts.total_attempts, correct: counts.correct, incorrect: counts.incorrect });
  }
  tiles.sort((a, b) => b.total_attempts - a.total_attempts || a.symbol.codePointAt(0)! - b.symbol.codePointAt(0)!);
  const remaining = { total_attempts: 0, correct: 0, incorrect: 0,
    symbolCount: Math.max(0, tiles.length - MAX_HEATMAP_TILES) };
  for (const tile of tiles.slice(MAX_HEATMAP_TILES)) {
    remaining.total_attempts += tile.total_attempts;
    remaining.correct += tile.correct;
    remaining.incorrect += tile.incorrect;
  }
  return { tiles: tiles.slice(0, MAX_HEATMAP_TILES), symbolCount: tiles.length, totals, remaining };
}

export function characterLabel(symbol: string, space: string): string {
  if (symbol === ' ') return space;
  if (/\p{Mark}/u.test(symbol)) return `◌${symbol}`;
  if (/[\p{Control}\p{Format}\p{Separator}]/u.test(symbol)) return '·';
  return symbol;
}
