// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors
// Test-only entrypoint. Build this HTML fixture with Vite; never import it from the app.
import { renderShareCard } from '../../frontend/src/lib/shareCard.ts';

const colors = { background: '#0d0f12', surface: '#15181d', text: '#e7e9ed', sub: '#8c94a0', accent: '#c5cbd4', error: '#dc8d8d' };
const stats = { wpm: 87.4, raw_wpm: 94.2, accuracy: 96.8, duration_ms: 30_000, mode: 'Время Čas Ćwiczenie', language: 'ru', date: '2026-10-17', heatmap: {} };
const check = (condition, message) => { if (!condition) throw new Error(message); };
const decode = async blob => {
  // Avoid blob: image URLs: production CSP allows only self, including fonts.
  const bitmap = await createImageBitmap(blob);
  const canvas = document.createElement('canvas');
  canvas.width = bitmap.width; canvas.height = bitmap.height;
  const ctx = canvas.getContext('2d'); ctx.drawImage(bitmap, 0, 0); bitmap.close();
  return { canvas, ctx };
};
const painted = (ctx, x, y, w, h, background) => {
  const pixels = ctx.getImageData(x, y, w, h).data;
  let count = 0;
  for (let i = 0; i < pixels.length; i += 4) if (pixels[i] !== background[0] || pixels[i + 1] !== background[1] || pixels[i + 2] !== background[2]) count++;
  return count;
};
try {
  const png = await renderShareCard(stats, colors);
  const { canvas, ctx } = await decode(png);
  check(canvas.width === 900 && canvas.height === 470, 'PNG dimensions');
  check(painted(ctx, 48, 80, 804, 35, [13, 15, 18]) > 300, 'Unicode subtitle visible');
  check(painted(ctx, 48, 200, 804, 100, [13, 15, 18]) > 1000, 'Scores visible');
  check(document.fonts.check('400 18px RacoonShareCard', 'Время Čas Ćwiczenie'), 'regular bundled font loaded');
  check(document.fonts.check('700 34px RacoonShareCard'), 'bold bundled font loaded');
  // Adding Unicode must change the rendered subtitle, rather than silently omit it.
  const withoutSubtitle = await decode(await renderShareCard({ ...stats, mode: '', language: '' }, colors));
  const first = ctx.getImageData(48, 80, 804, 35).data;
  const second = withoutSubtitle.ctx.getImageData(48, 80, 804, 35).data;
  check(first.some((value, i) => value !== second[i]), 'Unicode subtitle changes actual pixels');
  const light = await decode(await renderShareCard({ ...stats, wpm: 0, accuracy: 0, mode: 'Очень длинная подпись '.repeat(20) }, { ...colors, background: '#ffffff', accent: '#112233' }));
  check(light.ctx.getImageData(899, 469, 1, 1).data[0] === 255, 'theme background');
  check(light.ctx.getImageData(0, 0, 1, 1).data[0] === 17, 'theme accent');
  const fallback = await decode(await renderShareCard(stats, { ...colors, background: 'invalid' }));
  check(fallback.ctx.getImageData(899, 469, 1, 1).data[0] === 13, 'invalid color fallback');
  const heat = await decode(await renderShareCard({ ...stats, heatmap: { q: { total_attempts: 1, correct: 1 }, w: { total_attempts: 1, correct: 0 } } }, colors));
  check(heat.ctx.getImageData(60, 387, 1, 1).data[0] === 197, 'correct key accent');
  check(heat.ctx.getImageData(88, 387, 1, 1).data[0] === 220, 'incorrect key error color');
  document.getElementById('results').textContent = JSON.stringify({ ok: true, userAgent: navigator.userAgent, pngBytes: png.size, dimensions: [canvas.width, canvas.height], resources: performance.getEntriesByType('resource').map(entry => entry.name) });
  // The native harness reads the output from the DOM; this never reaches production.
  const bytes = new Uint8Array(await png.arrayBuffer());
  const output = document.createElement('pre'); output.id = 'png-base64'; output.hidden = true;
  output.textContent = btoa(Array.from(bytes, value => String.fromCharCode(value)).join(''));
  document.body.appendChild(output);
  document.body.appendChild(canvas);
  document.getElementById('status').textContent = 'PASS';
} catch (error) {
  document.getElementById('status').textContent = 'FAIL';
  document.getElementById('results').textContent = String(error);
}
