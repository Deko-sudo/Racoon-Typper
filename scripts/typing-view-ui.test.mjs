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
