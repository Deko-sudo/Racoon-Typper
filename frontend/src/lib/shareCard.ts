// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import regularFontUrl from '../../../resources/fonts/DejaVuSans.ttf?url';
import boldFontUrl from '../../../resources/fonts/DejaVuSans-Bold.ttf?url';
import type { FinalStats } from './types/index';

export type ShareCardStats = Pick<FinalStats, 'wpm' | 'raw_wpm' | 'accuracy' | 'duration_ms' | 'heatmap'> & {
  mode: string;
  language: string;
  date: string;
};
export type ShareCardColors = Record<'background' | 'surface' | 'text' | 'sub' | 'accent' | 'error', string>;
export const CARD_WIDTH = 900;
export const CARD_HEIGHT = 470;
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
export async function renderShareCard(stats: ShareCardStats, colors: ShareCardColors): Promise<Blob> {
  const canvas = document.createElement('canvas');
  canvas.width = CARD_WIDTH;
  canvas.height = CARD_HEIGHT;
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
    ctx.font = `${bold ? 700 : 400} ${size}px "${FONT_FAMILY}"`;
    ctx.fillStyle = color;
    ctx.fillText(value, x, y, Math.min(width, CARD_WIDTH - 48 - x));
  };
  ctx.fillStyle = pc.background;
  ctx.fillRect(0, 0, CARD_WIDTH, CARD_HEIGHT);
  ctx.fillStyle = pc.accent;
  ctx.fillRect(0, 0, CARD_WIDTH, 6);
  text('RACOON TYPPER', 48, 76, 34, pc.accent, true);
  text(`${stats.mode} · ${stats.language} · ${stats.date}`, 48, 108, 18, pc.sub);
  ctx.fillStyle = mix(pc.background, pc.sub, 0.25);
  ctx.fillRect(48, 140, CARD_WIDTH - 96, 1);
  ctx.fillRect(48, 330, CARD_WIDTH - 96, 1);
  const values = [stats.wpm.toFixed(1), stats.raw_wpm.toFixed(1), `${stats.accuracy.toFixed(1)}%`, `${(stats.duration_ms / 1000).toFixed(1)}s`];
  const labels = ['WPM', 'RAW WPM', 'ACCURACY', 'TIME'];
  values.forEach((value, i) => {
    text(value, 48 + 216 * i, 260, 56, pc.accent, true, 192);
    text(labels[i], 48 + 216 * i, 288, 20, pc.sub, false, 192);
  });
  text('KEYBOARD HEATMAP', 48, 366, 16, pc.sub);
  ['qwertyuiop', 'asdfghjkl', 'zxcvbnm'].forEach((row, r) => {
    [...row].forEach((key, c) => {
      const data = stats.heatmap[key];
      const used = data && data.total_attempts > 0;
      ctx.fillStyle = used
        ? mix(pc.error, pc.accent, data.correct / data.total_attempts)
        : mix(pc.surface, pc.sub, 0.15);
      const x = 48 + r * 14 + c * 28;
      const y = 384 + r * 28;
      // Preserve rounded keys using the baseline Canvas API (no roundRect).
      ctx.beginPath();
      ctx.moveTo(x + 5, y);
      ctx.lineTo(x + 19, y); ctx.quadraticCurveTo(x + 24, y, x + 24, y + 5);
      ctx.lineTo(x + 24, y + 19); ctx.quadraticCurveTo(x + 24, y + 24, x + 19, y + 24);
      ctx.lineTo(x + 5, y + 24); ctx.quadraticCurveTo(x, y + 24, x, y + 19);
      ctx.lineTo(x, y + 5); ctx.quadraticCurveTo(x, y, x + 5, y);
      ctx.fill();
      ctx.font = `400 16px "${FONT_FAMILY}"`;
      text(key, x + (24 - ctx.measureText(key).width) / 2, y + 18, 16, used ? pc.text : pc.sub, false, 24);
    });
  });
  const blob = await bounded(new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(value => value ? resolve(value) : reject(new Error('Share card PNG encoding failed')), 'image/png');
  }), 'Share card PNG encoding timed out');
  if (blob.type !== 'image/png' || !blob.size) throw new Error('Share card PNG encoding failed');
  return blob;
}

export function downloadShareCard(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  try {
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    try { link.click(); } finally { link.remove(); }
  } finally {
    // WebKit may consume the link asynchronously after click dispatch.
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
}
