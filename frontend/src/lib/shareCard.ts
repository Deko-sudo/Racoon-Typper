// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import regularFontUrl from '../../../resources/fonts/DejaVuSans.ttf?url';
import boldFontUrl from '../../../resources/fonts/DejaVuSans-Bold.ttf?url';
import type { FinalStats } from './types/index';
import { buildCharacterHeatmap, characterLabel } from './heatmapPresentation';

export type ShareCardStats = Pick<FinalStats, 'wpm' | 'raw_wpm' | 'accuracy' | 'duration_ms' | 'heatmap'> & {
  mode: string;
  language: string;
  date: string;
};
export type ShareCardColors = Record<'background' | 'surface' | 'text' | 'sub' | 'accent' | 'error', string>;
export const CARD_WIDTH = 900;
export const CARD_HEIGHT = 470;
export const CARD_MAX_HEIGHT = 900;
export interface ShareCardLabels {
  title: string;
  legend: string;
  empty: string;
  space: string;
  shown: string;
  remaining: string;
}
const DEFAULT_LABELS: ShareCardLabels = {
  title: 'Character accuracy', legend: 'Correct on the first attempt (%)',
  empty: 'No character data', space: 'Space', shown: 'Shown {shown} of {total} characters',
  remaining: 'Other {count}: {attempts} first attempts · {correct} correct · {incorrect} incorrect',
};
const FONT_FAMILY = 'RacoonShareCard';
const TIMEOUT_MS = 10_000;
let fontsReady: Promise<void> | undefined;

function bounded<T>(work: Promise<T>, message: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  return Promise.race([
    work,
    new Promise<never>((_, reject) => { timer = setTimeout(() => reject(new Error(message)), TIMEOUT_MS); }),
  ]).finally(() => clearTimeout(timer));
}

async function loadFonts(): Promise<void> {
  if (!fontsReady) {
    const regular = new FontFace(FONT_FAMILY, `url("${regularFontUrl}")`, { weight: '400' });
    const bold = new FontFace(FONT_FAMILY, `url("${boldFontUrl}")`, { weight: '700' });
    const attempt = bounded(Promise.all([regular.load(), bold.load()]), 'Share card fonts timed out')
      .then(([loadedRegular, loadedBold]) => {
        document.fonts.add(loadedRegular);
        document.fonts.add(loadedBold);
      });
    fontsReady = attempt;
    // Failed attempts may be retried; late font loads never register timed-out faces.
    void attempt.catch(() => { if (fontsReady === attempt) fontsReady = undefined; });
  }
  await fontsReady;
}

export function parseColor(value: string, fallback: string): string {
  const match = /^(?:#)?([\da-f]{6})(?:[\da-f]{2})?$/i.exec(value.trim());
  return match ? `#${match[1]}` : fallback;
}

function mix(a: string, b: string, amount: number): string {
  const t = Math.max(0, Math.min(1, amount));
  return `rgb(${[1, 3, 5].map(i => Math.round(
    parseInt(a.slice(i, i + 2), 16) * (1 - t) + parseInt(b.slice(i, i + 2), 16) * t,
  )).join(',')})`;
}

// Presentation only: scores and completion are supplied by the Rust backend.
export async function renderShareCard(stats: ShareCardStats, colors: ShareCardColors, labels: ShareCardLabels = DEFAULT_LABELS): Promise<Blob> {
  const heatmap = buildCharacterHeatmap(stats.heatmap);
  const gridY = 404, rowPitch = 68;
  const gridBottom = gridY + Math.ceil(heatmap.tiles.length / 10) * rowPitch;
  const canvas = document.createElement('canvas');
  canvas.width = CARD_WIDTH;
  canvas.height = Math.min(CARD_MAX_HEIGHT, Math.max(CARD_HEIGHT, gridBottom + (heatmap.remaining.symbolCount ? 64 : 28)));
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('Share card canvas is unavailable');
  await loadFonts();
  const pc = {
    background: parseColor(colors.background, '#0d0f12'),
    surface: parseColor(colors.surface, '#15181d'),
    text: parseColor(colors.text, '#e7e9ed'),
    sub: parseColor(colors.sub, '#8c94a0'),
    accent: parseColor(colors.accent, '#c5cbd4'),
    error: parseColor(colors.error, '#dc8d8d'),
  };
  const text = (value: string, x: number, y: number, size: number, color: string, bold = false, width = CARD_WIDTH - 48 - x) => {
    ctx.font = `${bold ? 700 : 400} ${size}px "${FONT_FAMILY}", sans-serif`;
    ctx.fillStyle = color;
    ctx.fillText(value, x, y, Math.min(width, CARD_WIDTH - 48 - x));
  };
  ctx.fillStyle = pc.background;
  ctx.fillRect(0, 0, CARD_WIDTH, canvas.height);
  ctx.fillStyle = pc.accent;
  ctx.fillRect(0, 0, CARD_WIDTH, 6);
  text('RACOON TYPPER', 48, 76, 34, pc.accent, true);
  text(`${stats.mode} · ${stats.language} · ${stats.date}`, 48, 108, 18, pc.sub);
  ctx.fillStyle = mix(pc.background, pc.sub, 0.25);
  ctx.fillRect(48, 140, CARD_WIDTH - 96, 1);
  ctx.fillRect(48, 330, CARD_WIDTH - 96, 1);
  const values = [stats.wpm.toFixed(1), stats.raw_wpm.toFixed(1), `${stats.accuracy.toFixed(1)}%`, `${(stats.duration_ms / 1000).toFixed(1)}s`];
  const metricLabels = ['WPM', 'RAW WPM', 'ACCURACY', 'TIME'];
  values.forEach((value, i) => {
    text(value, 48 + 216 * i, 260, 56, pc.accent, true, 192);
    text(metricLabels[i], 48 + 216 * i, 288, 20, pc.sub, false, 192);
  });
  text(labels.title, 48, 364, 18, pc.text, true);
  text(labels.legend, 48, 388, 14, pc.sub);
  if (!heatmap.tiles.length) text(labels.empty, 48, 430, 16, pc.sub);
  heatmap.tiles.forEach((tile, index) => {
    const x = 48 + (index % 10) * 80, y = gridY + Math.floor(index / 10) * rowPitch;
    const accuracy = tile.correct / tile.total_attempts;
    const color = mix(pc.error, pc.accent, accuracy);
    ctx.fillStyle = pc.surface;
    ctx.fillRect(x, y, 72, 60);
    ctx.fillStyle = color;
    ctx.fillRect(x, y + 58, 72, 2);
    const needsCodePoint = !/^[\x20-\x7e]$/.test(tile.symbol);
    text(characterLabel(tile.symbol, labels.space), x + 6, y + (needsCodePoint ? 23 : 30), 22, pc.text, false, 60);
    if (needsCodePoint) text(tile.codePoint, x + 6, y + 37, 10, pc.sub, false, 60);
    text(`${(accuracy * 100).toFixed(0)}%`, x + 6, y + 53, 13, pc.text, false, 60);
  });
  if (heatmap.remaining.symbolCount) {
    const format = (template: string, values: Record<string, number>) =>
      template.replace(/\{(\w+)\}/g, (match, key: string) => key in values ? String(values[key]) : match);
    text(format(labels.shown, { shown: heatmap.tiles.length, total: heatmap.symbolCount }), 48, gridBottom + 20, 14, pc.text);
    text(format(labels.remaining, { count: heatmap.remaining.symbolCount, attempts: heatmap.remaining.total_attempts,
      correct: heatmap.remaining.correct, incorrect: heatmap.remaining.incorrect }), 48, gridBottom + 42, 14, pc.sub);
  }
  const blob = await bounded(new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(value => value ? resolve(value) : reject(new Error('Share card PNG encoding failed')), 'image/png');
  }), 'Share card PNG encoding timed out');
  if (blob.type !== 'image/png' || !blob.size) throw new Error('Share card PNG encoding failed');
  return blob;
}
