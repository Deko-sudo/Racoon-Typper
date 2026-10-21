// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors
// Test-only entrypoint. Build this HTML fixture with Vite; never import it from the app.
import { renderShareCard } from '../../frontend/src/lib/shareCard.ts';
import { t, UI_LANGUAGES } from '../../frontend/src/lib/i18n.ts';
import { languageSamples, sampleHeatmap } from './share-card-languages.mjs';

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
  const heat = await decode(await renderShareCard({ ...stats, heatmap: { q: { total_attempts: 1, correct: 1, incorrect: 0 }, w: { total_attempts: 1, correct: 0, incorrect: 1 } } }, colors));
  check(heat.ctx.getImageData(60, 463, 1, 1).data[0] === 197, 'correct character accent');
  check(heat.ctx.getImageData(140, 463, 1, 1).data[0] === 220, 'incorrect character error color');
  check(Object.keys(languageSamples).sort().join() === UI_LANGUAGES.map(([lang]) => lang).sort().join(), 'all selectable languages covered');
  const cards = [];
  const labelsFor = lang => Object.fromEntries(Object.entries({ title:'symbols',legend:'first_attempt',empty:'empty',space:'space',shown:'shown',remaining:'remaining' })
    .map(([name, key]) => [name, t(lang, `share_card.${key}`)]));
  const encode = async blob => btoa(Array.from(new Uint8Array(await blob.arrayBuffer()), value => String.fromCharCode(value)).join(''));
  for (const [language, sample] of Object.entries(languageSamples)) {
    const heatmap = sampleHeatmap(sample);
    const card = await renderShareCard({ ...stats, mode:'custom', language, heatmap }, colors, labelsFor(language));
    const raster = await decode(card);
    check(raster.canvas.width === 900 && raster.canvas.height >= 470 && raster.canvas.height <= 900, `${language}: bounded PNG`);
    const symbols = Object.keys(heatmap).sort((a,b) => heatmap[b].total_attempts - heatmap[a].total_attempts || a.codePointAt(0) - b.codePointAt(0));
    symbols.forEach((symbol, index) => {
      const x = 48 + index % 10 * 80, y = 404 + Math.floor(index / 10) * 68;
      check(raster.ctx.getImageData(x + 10, y + 59, 1, 1).data[0] === 197, `${language}/${symbol}: painted tile`);
      // A visible glyph OR its code-point label is required; system fallback
      // glyph shape itself remains platform-specific and is reviewed visually.
      const tileBackground = raster.ctx.getImageData(x + 70, y + 2, 1, 1).data;
      check(tileBackground[0] === 21 && tileBackground[1] === 24 && tileBackground[2] === 29, `${language}/${symbol}: valid surface color`);
      check(painted(raster.ctx, x + 4, y + 2, 64, 52, tileBackground) > 20, `${language}/${symbol}: readable tile contents`);
    });
    cards.push({ language, symbols, dimensions:[raster.canvas.width,raster.canvas.height], bytes:card.size, png:await encode(card) });
    const caption = document.createElement('p'); caption.textContent = `${language}: ${sample}`;
    document.body.append(caption, raster.canvas);
  }
  const overflow = await renderShareCard({ ...stats, heatmap:sampleHeatmap(Array.from({length:1000},(_,i)=>String.fromCodePoint(0x4e00+i)).join('')) }, colors, labelsFor('ru'));
  const overflowRaster = await decode(overflow);
  check(overflowRaster.canvas.height <= 900, 'large character set has bounded Canvas');
  check(painted(overflowRaster.ctx, 48, 812, 804, 52, [13,15,18]) > 300, 'explicit remainder rendered');
  cards.push({ language:'overflow', dimensions:[overflowRaster.canvas.width,overflowRaster.canvas.height], bytes:overflow.size, png:await encode(overflow) });
  const matrix = document.createElement('pre'); matrix.id='png-cards'; matrix.hidden=true; matrix.textContent=JSON.stringify(cards); document.body.appendChild(matrix);
  document.getElementById('results').textContent = JSON.stringify({ ok: true, userAgent: navigator.userAgent, pngBytes: png.size, dimensions: [canvas.width, canvas.height], languageCards:15, resources: performance.getEntriesByType('resource').map(entry => entry.name) });
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
