// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const ts = require('typescript');
const source = readFileSync(new URL('../frontend/src/lib/shareCard.ts', import.meta.url), 'utf8');
let sequence = 0;
async function renderer() {
  const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText
    .replace(/import regularFontUrl[^;]+;/, 'const regularFontUrl = "/assets/DejaVuSans.ttf";')
    .replace(/import boldFontUrl[^;]+;/, 'const boldFontUrl = "/assets/DejaVuSans-Bold.ttf";');
  return import(`data:text/javascript;base64,${Buffer.from(js + `\n// instance ${sequence++}`).toString('base64')}`);
}
const stats = { wpm: 87.4, raw_wpm: 94.2, accuracy: 96.8, duration_ms: 30_000, mode: 'Время Čas', language: 'ru', date: '2026-10-17', heatmap: {} };
const colors = { background: '#0d0f12', surface: '#15181d', text: '#e7e9ed', sub: '#8c94a0', accent: '#c5cbd4', error: '#dc8d8d' };

function environment(t) {
  const env = { calls: [], faces: [], fonts: [], canvases: [], links: [], noContext: false, fontFailure: false, fontHang: false, nullBlob: false, encodingHang: false, clickFailure: false };
  const replace = (name, value) => {
    const old = Object.getOwnPropertyDescriptor(globalThis, name);
    Object.defineProperty(globalThis, name, { configurable: true, writable: true, value });
    t.after(() => old ? Object.defineProperty(globalThis, name, old) : delete globalThis[name]);
  };
  replace('FontFace', class {
    constructor(family, url, options) { Object.assign(this, { family, url, options }); env.faces.push(this); }
    load() { return env.fontHang ? new Promise(() => {}) : env.fontFailure ? Promise.reject(new Error('font failed')) : Promise.resolve(this); }
  });
  replace('document', {
    fonts: { add: face => env.fonts.push(face) },
    body: { appendChild: link => env.calls.push(['append', link]) },
    createElement(tag) {
      if (tag === 'a') {
        const link = { click() { env.calls.push(['click']); if (env.clickFailure) throw new Error('click failed'); }, remove() { env.calls.push(['remove']); } };
        env.links.push(link); return link;
      }
      assert.equal(tag, 'canvas');
      const ctx = {
        fillRect(...args) { env.calls.push(['rect', this.fillStyle, ...args]); },
        fillText(...args) { env.calls.push(['text', this.font, this.fillStyle, ...args]); },
        measureText: value => ({ width: value.length * 8 }),
        beginPath() {}, moveTo(x, y) { this.keyPosition = [x - 5, y]; },
        lineTo() {}, quadraticCurveTo() {},
        fill() { env.calls.push(['key', this.fillStyle, ...this.keyPosition]); },
      };
      const canvas = {
        getContext: type => { assert.equal(type, '2d'); return env.noContext ? null : ctx; },
        toBlob(callback, type) { assert.equal(type, 'image/png'); if (!env.encodingHang) callback(env.nullBlob ? null : new Blob(['unit fixture'], { type })); },
      };
      env.canvases.push(canvas); return canvas;
    },
  });
  return env;
}

test('draws backend values, Unicode, theme and empty heatmap at 900x470 using bundled fonts', async t => {
  const env = environment(t); const { renderShareCard } = await renderer();
  const blob = await renderShareCard(Object.freeze(stats), Object.freeze(colors));
  assert.equal(blob.type, 'image/png');
  assert.equal(env.canvases[0].width, 900); assert.equal(env.canvases[0].height, 470);
  const text = env.calls.filter(c => c[0] === 'text').map(c => c[3]);
  for (const value of ['87.4', '94.2', '96.8%', '30.0s', 'Время Čas · ru · 2026-10-17', 'WPM', 'TIME']) assert.ok(text.includes(value), value);
  assert.equal(text.filter(v => /^[a-z]$/.test(v)).length, 26);
  assert.equal(env.fonts.length, 2);
  assert.deepEqual(env.faces.map(f => f.options.weight), ['400', '700']);
  assert.ok(env.faces.every(f => /^url\("\/assets\//.test(f.url)));
  assert.deepEqual(env.calls[0], ['rect', colors.background, 0, 0, 900, 470]);
  await renderShareCard({ ...stats, wpm: 0, accuracy: 0 }, colors);
  assert.equal(env.faces.length, 2, 'loaded fonts are reused');
});

test('color fallback, heatmap accuracy and long labels preserve card bounds', async t => {
  const env = environment(t); const { renderShareCard, parseColor } = await renderer();
  assert.equal(parseColor('#00ff0080', '#ffffff'), '#00ff00');
  assert.equal(parseColor('ff0000', '#ffffff'), '#ff0000');
  assert.equal(parseColor('invalid', '#ffffff'), '#ffffff');
  await renderShareCard({ ...stats, mode: 'я'.repeat(200), heatmap: { q: { total_attempts: 2, correct: 2 }, w: { total_attempts: 2, correct: 0 } } }, { ...colors, background: 'invalid' });
  assert.equal(env.calls[0][1], '#0d0f12');
  assert.ok(env.calls.some(c => c[0] === 'key' && c[1] === 'rgb(197,203,212)' && c[2] === 48 && c[3] === 384));
  assert.ok(env.calls.some(c => c[0] === 'key' && c[1] === 'rgb(220,141,141)' && c[2] === 76 && c[3] === 384));
  assert.ok(env.calls.filter(c => c[0] === 'text').every(c => c[6] > 0 && c[4] + c[6] <= 876));
});

test('context, font and encoding failures reject; failed font loading can be retried', async t => {
  const env = environment(t); const { renderShareCard } = await renderer();
  env.noContext = true; await assert.rejects(renderShareCard(stats, colors), /canvas/);
  env.noContext = false; env.fontFailure = true; await assert.rejects(renderShareCard(stats, colors), /font failed/);
  assert.equal(env.fonts.length, 0);
  env.fontFailure = false; env.nullBlob = true; await assert.rejects(renderShareCard(stats, colors), /encoding failed/);
  env.nullBlob = false; assert.equal((await renderShareCard(stats, colors)).type, 'image/png');
});

test('hung font loading times out and a subsequent export retries', async t => {
  const env = environment(t); const { renderShareCard } = await renderer();
  t.mock.timers.enable({ apis: ['setTimeout'] });
  env.fontHang = true;
  const attempt = assert.rejects(renderShareCard(stats, colors), /fonts timed out/);
  t.mock.timers.tick(10_001); await attempt;
  assert.equal(env.fonts.length, 0);
  env.fontHang = false; await renderShareCard(stats, colors);
  assert.equal(env.fonts.length, 2);
});

test('hung PNG encoding times out instead of leaving the export pending', async t => {
  const env = environment(t); const { renderShareCard } = await renderer();
  await renderShareCard(stats, colors);
  t.mock.timers.enable({ apis: ['setTimeout'] }); env.encodingHang = true;
  const attempt = assert.rejects(renderShareCard(stats, colors), /encoding timed out/);
  // Cached fonts still resume through an await before scheduling encoding.
  await Promise.resolve(); await Promise.resolve(); await Promise.resolve();
  t.mock.timers.tick(10_001); await attempt;
});

test('download retains its URL through click and cleans up on success and failure', async t => {
  const env = environment(t); const { downloadShareCard } = await renderer();
  const revoke = t.mock.method(URL, 'revokeObjectURL', () => {});
  t.mock.timers.enable({ apis: ['setTimeout'] });
  downloadShareCard(new Blob(['png']), 'card.png');
  assert.equal(env.links[0].download, 'card.png');
  assert.deepEqual(env.calls.map(c => c[0]), ['append', 'click', 'remove']);
  assert.equal(revoke.mock.callCount(), 0);
  t.mock.timers.tick(1000); assert.equal(revoke.mock.callCount(), 1);
  env.clickFailure = true;
  assert.throws(() => downloadShareCard(new Blob(['png']), 'card.png'), /click failed/);
  t.mock.timers.tick(1000); assert.equal(revoke.mock.callCount(), 2);
  assert.equal(env.calls.at(-1)[0], 'remove');
});

test('production PNG path removes the font parser and obsolete IPC permission', () => {
  const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
  assert.doesNotMatch(read('crates/app/Cargo.toml'), /\bab_glyph\b/);
  assert.doesNotMatch(read('Cargo.lock'), /name = "(?:ab_glyph|ab_glyph_rasterizer|owned_ttf_parser|ttf-parser)"/);
  for (const path of ['crates/app/src/main.rs', 'crates/app/build.rs', 'crates/app/src/commands/reporting.rs', 'frontend/src/lib/api/ipc.ts']) {
    assert.doesNotMatch(read(path), /\bexport_result_png\b|\bshare_card\b/);
  }
  assert.doesNotMatch(read('crates/app/capabilities/main.json'), /allow-export-result-png/);
  const overlay = read('frontend/src/components/ResultOverlay.svelte');
  assert.match(overlay, /renderShareCard/); assert.doesNotMatch(overlay, /ipc\.exportResultPng/);
  assert.match(overlay, /if \(shareState === 'rendering'\) return/);
  assert.match(read('crates/app/tauri.conf.json'), /\.\.\/\.\.\/resources\/fonts\/DejaVu-LICENSE/);
  assert.doesNotMatch(source, /\bfetch\s*\(|https?:\/\/|innerHTML|\.invoke\(/);
});

test('actual result handler prevents overlapping exports and recovers after renderer failure', async t => {
  const overlay = readFileSync(new URL('../frontend/src/components/ResultOverlay.svelte', import.meta.url), 'utf8');
  const script = overlay.split('<script lang="ts">')[1].split('</script>')[0];
  const ast = ts.createSourceFile('ResultOverlay.ts', script, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  const functions = ast.statements.filter(node => ts.isFunctionDeclaration(node) && ['themeCssVar', 'handleSharePng'].includes(node.name?.text));
  assert.equal(functions.length, 2);
  const compiled = ts.transpileModule(functions.map(node => node.getText(ast)).join('\n'), {
    compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
  }).outputText;
  let release; let renders = 0; let downloads = 0; let received;
  const render = value => { renders++; received = value; return new Promise(resolve => { release = resolve; }); };
  const controller = new Function('renderShareCard', 'downloadShareCard', 'stats', 'getComputedStyle', 'document', `
    let shareState = 'idle', sessionModeType = 'custom', sessionLanguage = 'ru';
    ${compiled}
    return { run: handleSharePng, state: () => shareState };
  `)(render, () => { downloads++; }, stats, () => ({ getPropertyValue: () => '' }), { documentElement: {} });
  const first = controller.run(); await controller.run();
  assert.equal(renders, 1); assert.equal(controller.state(), 'rendering');
  assert.equal(received.wpm, stats.wpm); assert.equal(received.language, 'ru');
  release(new Blob(['png'])); await first;
  assert.equal(downloads, 1); assert.equal(controller.state(), 'idle');
  const failure = new Function('renderShareCard', 'downloadShareCard', 'stats', 'getComputedStyle', 'document', `
    let shareState = 'idle', sessionModeType = 'custom', sessionLanguage = 'ru';
    ${compiled}
    return { run: handleSharePng, state: () => shareState };
  `)(async () => { throw new Error('font failed'); }, () => assert.fail('must not download on failure'), stats, () => ({ getPropertyValue: () => '' }), { documentElement: {} });
  const errors = t.mock.method(console, 'error', () => {});
  await failure.run(); assert.equal(failure.state(), 'error');
  await failure.run(); assert.equal(errors.mock.callCount(), 2, 'error state allows retry');
});
