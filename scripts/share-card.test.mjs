// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { languageSamples, sampleHeatmap } from './fixtures/share-card-languages.mjs';
import { t as translate, UI_LANGUAGES } from './i18n-test-loader.mjs';
const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const ts = require('typescript');
const asModule = js => `data:text/javascript;base64,${Buffer.from(js).toString('base64')}`;
const modelSource = readFileSync(new URL('../frontend/src/lib/heatmapPresentation.ts', import.meta.url), 'utf8');
const modelUrl = asModule(ts.transpileModule(modelSource, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText);
const { buildCharacterHeatmap, characterLabel, MAX_HEATMAP_TILES } = await import(modelUrl);
const source = readFileSync(new URL('../frontend/src/lib/shareCard.ts', import.meta.url), 'utf8');
let sequence = 0;
async function renderer() {
  const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText
    .replace(/import regularFontUrl[^;]+;/, 'const regularFontUrl = "/assets/DejaVuSans.ttf";')
    .replace(/import boldFontUrl[^;]+;/, 'const boldFontUrl = "/assets/DejaVuSans-Bold.ttf";')
    .replace("'./heatmapPresentation'", JSON.stringify(modelUrl));
  return import(`data:text/javascript;base64,${Buffer.from(js + `\n// instance ${sequence++}`).toString('base64')}`);
}
const stats = { wpm: 87.4, raw_wpm: 94.2, accuracy: 96.8, duration_ms: 30_000, mode: 'Время Čas', language: 'ru', date: '2026-10-17', heatmap: {} };
const colors = { background: '#0d0f12', surface: '#15181d', text: '#e7e9ed', sub: '#8c94a0', accent: '#c5cbd4', error: '#dc8d8d' };
const labelKeys = { title: 'symbols', legend: 'first_attempt', empty: 'empty', space: 'space', shown: 'shown', remaining: 'remaining' };
const labelsFor = lang => Object.fromEntries(Object.entries(labelKeys).map(([name, key]) => [name, translate(lang, `share_card.${key}`)]));

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

for (const [language, sample] of Object.entries(languageSamples)) {
  test(`PNG preserves every expected letter in ${language}`, async t => {
    const env = environment(t); const { renderShareCard } = await renderer();
    await renderShareCard({ ...stats, language, heatmap: sampleHeatmap(sample) }, colors, labelsFor(language));
    const labels = env.calls.filter(c => c[0] === 'text').map(c => c[3]);
    for (const symbol of new Set(Array.from(sample).filter(ch => /\p{Letter}/u.test(ch)))) {
      assert.ok(labels.includes(symbol), `${language}: missing ${symbol}`);
    }
    assert.ok(labels.includes(translate(language, 'share_card.symbols')));
    assert.ok(labels.includes(translate(language, 'share_card.first_attempt')));
    assert.ok(labels.includes(translate(language, 'share_card.space')));
    const canvas = env.canvases[0];
    assert.equal(canvas.width, 900); assert.ok(canvas.height >= 470 && canvas.height <= 900);
    assert.equal(env.calls.filter(c => c[0] === 'rect' && c[4] === 72 && c[5] === 60).length, new Set(Array.from(sample)).size);
    assert.ok(env.calls.filter(c => c[0] === 'rect' && c[4] === 72 && c[5] === 60).every(c => c[1] === colors.surface));
  });
}

test('fixtures and explicit PNG translations cover every selectable UI language', () => {
  assert.deepEqual(Object.keys(languageSamples).sort(), UI_LANGUAGES.map(([lang]) => lang).sort());
  const ast = ts.createSourceFile('i18n.ts', readFileSync(new URL('../frontend/src/lib/i18n.ts', import.meta.url), 'utf8'), ts.ScriptTarget.Latest, true);
  const variables = ast.statements.filter(ts.isVariableStatement).flatMap(s => [...s.declarationList.declarations]);
  for (const [lang] of UI_LANGUAGES) {
    const name = { 'zh-hk': 'zhHK', 'zh-tw': 'zhTW' }[lang] ?? lang;
    const properties = new Set(variables.find(v => v.name.text === name).initializer.properties.map(p => p.name.text));
    for (const key of Object.values(labelKeys)) {
      assert.ok(properties.has(`share_card.${key}`), `${lang}/${key}: requires explicit translation`);
      const placeholders = value => [...value.matchAll(/\{(\w+)\}/g)].map(m => m[1]).sort();
      assert.deepEqual(placeholders(translate(lang, `share_card.${key}`)), placeholders(translate('en', `share_card.${key}`)));
    }
  }
});

test('character counts retain case, diacritics and scalar identities without alias duplication', () => {
  const input = sampleHeatmap('Aaаёїґößș漢한😀𠮷ée\u0301 1!?');
  for (const value of Object.values(input)) Object.freeze(value);
  Object.freeze(input);
  const model = buildCharacterHeatmap(input);
  assert.equal(model.symbolCount, Object.keys(input).length);
  for (const [symbol, counts] of Object.entries(input)) {
    const tile = model.tiles.find(t => t.symbol === symbol);
    for (const field of ['correct', 'incorrect', 'total_attempts']) assert.equal(tile[field], counts[field], symbol);
  }
  assert.equal(model.tiles.find(t => t.symbol === 'A').total_attempts, 1);
  assert.equal(model.tiles.find(t => t.symbol === 'a').total_attempts, 1);
  assert.equal(model.tiles.find(t => t.symbol === '𠮷').codePoint, 'U+20BB7');
  assert.equal(characterLabel(' ', 'Пробел'), 'Пробел');
  assert.equal(characterLabel('\u0301', 'Space'), '◌\u0301');
  assert.equal(characterLabel('\u200d', 'Space'), '·');
  assert.equal(characterLabel('\n', 'Space'), '·');
});

test('empty and invalid counts never produce misleading percentages or mutate the input', () => {
  assert.deepEqual(buildCharacterHeatmap({}).tiles, []);
  assert.deepEqual(buildCharacterHeatmap({ a: { total_attempts: 0, correct: 0, incorrect: 0 } }).tiles, []);
  for (const counts of [null, { total_attempts: -1, correct: 0, incorrect: 0 },
    { total_attempts: 1, correct: 0.5, incorrect: 0.5 }, { total_attempts: NaN, correct: 0, incorrect: 0 },
    { total_attempts: 1, correct: 1, incorrect: 1 }]) {
    assert.throws(() => buildCharacterHeatmap({ a: counts }), /Invalid character heatmap/);
  }
  assert.throws(() => buildCharacterHeatmap({ '\ud800': { total_attempts: 1, correct: 1, incorrect: 0 } }), /Invalid character heatmap/);
  assert.throws(() => buildCharacterHeatmap({ aa: { total_attempts: 1, correct: 1, incorrect: 0 } }), /Invalid character heatmap/);
  assert.throws(() => buildCharacterHeatmap({ a: { total_attempts: Number.MAX_SAFE_INTEGER, correct: Number.MAX_SAFE_INTEGER, incorrect: 0 }, b: { total_attempts: 1, correct: 1, incorrect: 0 } }), /supported range/);
});

test('tile limit conserves every count in the explicit remainder and order is deterministic', () => {
  for (const size of [59, 60, 61, 1000]) {
    const input = Object.fromEntries(Array.from({ length: size }, (_, i) => {
      const total_attempts = 1 + i % 7, correct = Math.floor(total_attempts / 2);
      return [String.fromCodePoint(0x4e00 + i), { total_attempts, correct, incorrect: total_attempts - correct }];
    }));
    const model = buildCharacterHeatmap(input);
    assert.equal(model.tiles.length, Math.min(MAX_HEATMAP_TILES, size));
    assert.equal(model.remaining.symbolCount, Math.max(0, size - MAX_HEATMAP_TILES));
    for (const field of ['correct', 'incorrect', 'total_attempts']) {
      const total = Object.values(input).reduce((n, value) => n + value[field], 0);
      assert.equal(model.totals[field], total);
      assert.equal(model.tiles.reduce((n, tile) => n + tile[field], model.remaining[field]), total);
    }
    assert.deepEqual(model, buildCharacterHeatmap(Object.fromEntries(Object.entries(input).reverse())));
  }
});

test('overflow PNG is bounded and accounts for the undisplayed symbols explicitly', async t => {
  const env = environment(t); const { renderShareCard } = await renderer();
  const heatmap = sampleHeatmap(Array.from({ length: 1000 }, (_, i) => String.fromCodePoint(0x4e00 + i)).join(''));
  await renderShareCard({ ...stats, heatmap }, colors, labelsFor('ru'));
  const canvas = env.canvases[0]; assert.ok(canvas.height <= 900);
  const text = env.calls.filter(c => c[0] === 'text').map(c => c[3]);
  assert.ok(text.includes('Показано 60 из 1000 символов'));
  assert.ok(text.includes('Остальные 940: первых попыток 940 · верно 940 · ошибок 0'));
  assert.equal(text.filter(s => /^U\+[0-9A-F]+$/.test(s)).length, 60);
  assert.ok(env.calls.filter(c => c[0] === 'text').every(c => c[4] + c[6] <= 876 && c[5] < canvas.height));
});

test('non-ASCII and invisible symbols keep an ASCII code-point caption without replacing the glyph', async t => {
  const env = environment(t); const { renderShareCard } = await renderer();
  await renderShareCard({ ...stats, heatmap: sampleHeatmap('Aé e\u0301😀𠮷\u200d') }, colors);
  const text = env.calls.filter(c => c[0] === 'text').map(c => c[3]);
  for (const value of ['A', 'é', 'e', '◌\u0301', '😀', '𠮷', '·', 'U+00E9', 'U+0301', 'U+1F600', 'U+20BB7', 'U+200D']) assert.ok(text.includes(value), value);
  assert.equal(text.includes('U+0041'), false, 'ordinary ASCII needs no technical caption');
});

test('draws backend values, Unicode, theme and empty heatmap at 900x470 using bundled fonts', async t => {
  const env = environment(t); const { renderShareCard } = await renderer();
  const blob = await renderShareCard(Object.freeze(stats), Object.freeze(colors));
  assert.equal(blob.type, 'image/png');
  assert.equal(env.canvases[0].width, 900); assert.equal(env.canvases[0].height, 470);
  const text = env.calls.filter(c => c[0] === 'text').map(c => c[3]);
  for (const value of ['87.4', '94.2', '96.8%', '30.0s', 'Время Čas · ru · 2026-10-17', 'WPM', 'TIME']) assert.ok(text.includes(value), value);
  assert.ok(text.includes('No character data'));
  assert.equal(text.filter(v => /^[a-z]$/.test(v)).length, 0);
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
  await renderShareCard({ ...stats, mode: 'я'.repeat(200), heatmap: { q: { total_attempts: 2, correct: 2, incorrect: 0 }, w: { total_attempts: 2, correct: 0, incorrect: 2 } } }, { ...colors, background: 'invalid' });
  assert.equal(env.calls[0][1], '#0d0f12');
  assert.ok(env.calls.some(c => c[0] === 'rect' && c[1] === 'rgb(197,203,212)' && c[2] === 48 && c[3] === 462));
  assert.ok(env.calls.some(c => c[0] === 'rect' && c[1] === 'rgb(220,141,141)' && c[2] === 128 && c[3] === 462));
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
  assert.match(overlay, /if \(shareState === 'rendering' \|\| shareState === 'saving'\) return/);
  assert.match(read('crates/app/tauri.conf.json'), /\.\.\/\.\.\/resources\/fonts\/DejaVu-LICENSE/);
  assert.doesNotMatch(source, /\bfetch\s*\(|https?:\/\/|innerHTML|\.invoke\(/);
});


function resultController(render, save) {
  const overlay = readFileSync(new URL('../frontend/src/components/ResultOverlay.svelte', import.meta.url), 'utf8');
  const script = overlay.split('<script lang="ts">')[1].split('</script>')[0];
  const ast = ts.createSourceFile('ResultOverlay.ts', script, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  const functions = ast.statements.filter(node => ts.isFunctionDeclaration(node) && ['themeCssVar', 'handleSharePng'].includes(node.name?.text));
  assert.equal(functions.length, 2);
  const compiled = ts.transpileModule(functions.map(node => node.getText(ast)).join('\n'), {
    compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
  }).outputText;
  return new Function('renderShareCard', 'saveResultCard', 'stats', 'getComputedStyle', 'document', 't', `
    let shareState = 'idle', shareOutcome = null, sessionModeType = 'custom', sessionLanguage = 'ru', uiLang = 'de';
    ${compiled}
    return { run: handleSharePng, state: () => shareState, outcome: () => shareOutcome };
  `)(render, save, stats, () => ({ getPropertyValue: () => '' }), { documentElement: {} }, translate);
}

test('actual result handler waits for OS acknowledgement and prevents overlapping rendering and delivery', async () => {
  let releaseRender, releaseSave; let renders = 0; let saves = 0; let received, receivedLabels;
  const controller = resultController((value, _, labels) => {
    renders++; received = value; receivedLabels = labels; return new Promise(resolve => { releaseRender = resolve; });
  }, bytes => { saves++; assert.deepEqual(bytes, [1, 2, 3]); return new Promise(resolve => { releaseSave = resolve; }); });
  const first = controller.run(); await controller.run();
  assert.equal(renders, 1); assert.equal(controller.state(), 'rendering');
  assert.equal(received.wpm, stats.wpm); assert.equal(received.language, 'ru');
  assert.equal(receivedLabels.title, translate('de', 'share_card.symbols'));
  releaseRender(new Blob([new Uint8Array([1, 2, 3])]));
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(controller.state(), 'saving'); assert.equal(controller.outcome(), null);
  await controller.run(); assert.equal(renders, 1); assert.equal(saves, 1);
  releaseSave({ savedPath: '/Pictures/Raccoon Typer/card.png', copied: true }); await first;
  assert.equal(controller.state(), 'done'); assert.equal(controller.outcome().copied, true);
});

for (const outcome of [
  { savedPath: '/Pictures/Raccoon Typer/card.png', copied: true },
  { savedPath: '/Pictures/Raccoon Typer/card.png', copied: false },
  { savedPath: null, copied: true }, { savedPath: null, copied: false },
]) {
  test(`actual result handler retains independent delivery outcomes ${JSON.stringify(outcome)}`, async () => {
    let count = 0;
    const controller = resultController(async () => new Blob(['png']), async () => { count++; return outcome; });
    await controller.run(); assert.equal(controller.state(), 'done'); assert.equal(controller.outcome(), outcome);
    await controller.run(); assert.equal(count, 2, 'partial and complete failures allow retry');
  });
}

for (const stage of ['render', 'delivery']) {
  test(`actual result handler recovers from ${stage} failure without false success`, async t => {
    const errors = t.mock.method(console, 'error', () => {}); let fail = true;
    const controller = resultController(async () => {
      if (fail && stage === 'render') throw Error('/private/font');
      return new Blob(['png']);
    }, async () => {
      if (fail) throw Error('/private/Pictures');
      return { savedPath: '/Pictures/Raccoon Typer/card.png', copied: true };
    });
    await controller.run(); assert.equal(controller.state(), 'error'); assert.equal(controller.outcome(), null);
    assert.equal(errors.mock.calls[0].arguments.length, 1, 'no OS error details logged');
    fail = false; await controller.run(); assert.equal(controller.state(), 'done');
  });
}

test('PNG feedback is localized for every UI language and accessible without motion', () => {
  const overlay = readFileSync(new URL('../frontend/src/components/ResultOverlay.svelte', import.meta.url), 'utf8');
  assert.doesNotMatch(overlay, /downloadShareCard|createObjectURL/);
  assert.match(overlay, /role="status" aria-live="polite" aria-atomic="true"/);
  assert.match(overlay, /prefers-reduced-motion: reduce/);
  assert.match(overlay, /shareOutcome.savedPath/); assert.match(overlay, /shareOutcome.copied/);
  const dictionary = readFileSync(new URL('../frontend/src/lib/i18n.ts', import.meta.url), 'utf8');
  const ast = ts.createSourceFile('i18n.ts', dictionary, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  const declarations = ast.statements.filter(ts.isVariableStatement).flatMap(s => s.declarationList.declarations);
  for (const [lang] of UI_LANGUAGES) {
    const name = { 'zh-hk': 'zhHK', 'zh-tw': 'zhTW' }[lang] ?? lang;
    const properties = new Set(declarations.find(v => v.name.text === name).initializer.properties.map(p => p.name.text));
    for (const key of ['share_png', 'share_png_error', 'png_rendering', 'png_delivering', 'png_saved', 'png_copied', 'png_save_failed', 'png_copy_failed']) {
      assert.ok(properties.has(`result.${key}`), `${lang}/${key}: explicit translation`);
      assert.notEqual(translate(lang, `result.${key}`), `result.${key}`);
    }
  }
});
