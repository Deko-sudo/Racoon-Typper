// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { i18nUrl } from './i18n-test-loader.mjs';

const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const { compile } = require('svelte/compiler');
const ts = require('typescript');
const { render } = await import(pathToFileURL(require.resolve('svelte/server')).href);
const moduleUrl = source => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
const internal = JSON.stringify(pathToFileURL(require.resolve('svelte/internal/server')).href);
const compileComponent = (source, name) => compile(source, { filename: name, generate: 'server', dev: false }).js.code
  .replace("'svelte/internal/server'", internal);
// Child rendering is isolated: expose the exact nextChar prop from the real
// view, without re-testing the keyboard/hand components' own mapping.
const childUrl = moduleUrl(compileComponent('<script>let {nextChar = ""} = $props();</script><span data-next-char={nextChar}></span>', 'Child.svelte'));
const keyboardSource = readFileSync(new URL('../frontend/src/lib/keyboard.ts', import.meta.url), 'utf8');
const keyboardUrl = moduleUrl(ts.transpileModule(keyboardSource, {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText);
async function loadView(name) {
  const source = readFileSync(new URL(`../frontend/src/components/${name}.svelte`, import.meta.url), 'utf8');
  const compiled = compileComponent(source, `${name}.svelte`)
    .replaceAll(/'\.\/[^']+\.svelte'/g, JSON.stringify(childUrl))
    .replace("'../lib/i18n'", JSON.stringify(i18nUrl))
    .replace("'../lib/keyboard'", JSON.stringify(keyboardUrl));
  return (await import(moduleUrl(compiled))).default;
}
const TestView = await loadView('TestView');
const WeakKeysPanel = await loadView('WeakKeysPanel');
const KeyboardHeatmap = await loadView('KeyboardHeatmap');
const text = 'a😀б';
const statuses = Array.from(text, expected => ({ expected, typed: null, status: 'pending' }));
const settings = { show_keyboard_trainer: true, show_hand_guide: true, caret_style: 'off' };

test('TestView progress and next-key guidance share the backend Unicode-scalar positions', () => {
  for (const [caretPos, next] of [['0', 'a'], ['1', '😀'], ['2', 'б']]) {
    const markup = render(TestView, { props: {
      text, charStatuses: statuses, caretPos: Number(caretPos), isRunning: true, isComplete: false,
      liveWpm: 0, liveAccuracy: 100, elapsedMs: 0, finalStats: null, settings,
      selectedMode: 'custom', selectedLanguage: 'en', selectedDuration: 30, selectedWordCount: 25,
      sessionModeType: 'custom', sessionLanguage: 'en', lessonNavigation: null,
    } }).body;
    assert.ok(markup.includes(`${caretPos}/3`), `scalar progress at ${caretPos}`);
    assert.ok(markup.includes(`data-next-char="${next}"`), `whole next character at ${caretPos}`);
  }
});

test('inline weak-key guidance uses the same Unicode-scalar position as the rendered text', () => {
  for (const [trainingCaretPos, next] of [[1, '😀'], [2, 'б']]) {
    const markup = render(WeakKeysPanel, { props: {
      weakKeys: [], charStats: {}, trainingText: text, trainingCharStatuses: statuses,
      trainingRunning: true, trainingCaretPos,
    } }).body;
    assert.ok(markup.includes(`data-next-char="${next}"`), `whole next character at ${trainingCaretPos}`);
  }
});

function heatmapKey(key, props = {}) {
  const markup = render(KeyboardHeatmap, { props }).body;
  const tile = [...markup.matchAll(/<div\s+class="key(?:\s[^"]*)?"[^>]*>[\s\S]*?<\/div>/g)]
    .map(match => match[0]).find(html => html.includes(`title="${key}: `));
  assert.ok(tile, `keyboard contains ${key}`);
  return tile;
}
const counts = (correct, incorrect = 0) => Object.freeze({ correct, incorrect, total: correct + incorrect });
const attempts = (correct, incorrect = 0) => Object.freeze({ correct, incorrect,
  total_attempts: correct + incorrect, avg_wpm_at_key: 0 });

test('heatmap counts a lowercase source key once', () => {
  const tile = heatmapKey('a', { charStats: Object.freeze({ a: counts(1) }) });
  assert.match(tile, /title="a: 100%"/);
  assert.match(tile, /opacity:\s*0\.05[;"]/);
});

test('heatmap combines mixed-case counts with 50% accuracy instead of 67%', () => {
  for (const keyboardLayout of ['qwerty', 'dvorak']) {
    const tile = heatmapKey('a', { keyboardLayout, charStats: Object.freeze({ a: counts(1), A: counts(0, 1) }) });
    assert.match(tile, /title="a: 50%"/);
    assert.match(tile, /opacity:\s*0\.1[;"]/);
  }
});

test('heatmap accepts uppercase-only data once', () => {
  const tile = heatmapKey('a', { charStats: { A: counts(1) } });
  assert.match(tile, /title="a: 100%"/);
  assert.match(tile, /opacity:\s*0\.05[;"]/);
});

test('heatmap counts punctuation once when lower and upper aliases are identical', () => {
  const tile = heatmapKey(';', { charStats: { ';': counts(1, 1) } });
  assert.match(tile, /title=";: 50%"/);
  assert.match(tile, /opacity:\s*0\.1[;"]/);
});

test('heatmap combines Cyrillic case variants without a duplicate direct entry', () => {
  const tile = heatmapKey('ф', { charStats: { ф: counts(1), Ф: counts(0, 1) } });
  assert.match(tile, /title="ф: 50%"/);
  assert.match(tile, /opacity:\s*0\.1[;"]/);
});

test('heatmap fallback uses each exact first-attempt source key once', () => {
  const tile = heatmapKey('a', { heatmap: Object.freeze({ a: attempts(1), A: attempts(0, 1) }) });
  assert.match(tile, /title="a: 50%"/);
  assert.match(tile, /opacity:\s*0\.1[;"]/);
});

test('char stats override a matching heatmap key while a missing case can fall back', () => {
  const tile = heatmapKey('a', { charStats: { a: counts(1) }, heatmap: { a: attempts(0, 5), A: attempts(0, 1) } });
  assert.match(tile, /title="a: 50%"/);
  assert.match(tile, /opacity:\s*0\.1[;"]/);
});

test('distinct source keys are counted even when they share an object', () => {
  const same = counts(1);
  const tile = heatmapKey('a', { charStats: Object.freeze({ a: same, A: same }) });
  assert.match(tile, /title="a: 100%"/);
  assert.match(tile, /opacity:\s*0\.1[;"]/);
});

test('empty heatmap retains its unused-key state', () => {
  const tile = heatmapKey('a');
  assert.match(tile, /title="a: "/);
  assert.match(tile, /opacity:\s*0\.3[;"]/);
  assert.doesNotMatch(tile, /class="key-acc"/);
});
