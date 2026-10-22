// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { i18nUrl } from './i18n-test-loader.mjs';
import { HAND_FINGERS, HAND_OUTLINE } from '../frontend/src/lib/handGuide.ts';

// Render the actual Svelte component with the existing frontend compiler.
const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const { compile } = require('svelte/compiler');
const ts = require('typescript');
const { render } = await import(pathToFileURL(require.resolve('svelte/server')).href);
const moduleUrl = (source) => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
let compiled = compile(readFileSync(process.env.HAND_GUIDE_SOURCE ??
  new URL('../frontend/src/components/HandPositionGuide.svelte', import.meta.url), 'utf8'), {
  filename: 'HandPositionGuide.svelte', generate: 'server', dev: false,
}).js.code.replace("'svelte/internal/server'", JSON.stringify(pathToFileURL(require.resolve('svelte/internal/server')).href));
compiled = compiled.replace("'../lib/i18n'", JSON.stringify(i18nUrl));
for (const name of ['keyboard', 'handGuide']) {
  const source = readFileSync(new URL(`../frontend/src/lib/${name}.ts`, import.meta.url), 'utf8');
  const js = ts.transpileModule(source, {
    compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
  }).outputText;
  compiled = compiled.replace(`'../lib/${name}'`, JSON.stringify(moduleUrl(js)));
}
const { default: HandGuide } = await import(moduleUrl(compiled));
const html = (props) => render(HandGuide, { props }).body;
const fingers = (markup) => [...markup.matchAll(/<g\b([^>]*\bdata-finger="([A-Z]+)"[^>]*)>([\s\S]*?)<\/g>/g)];
const active = (markup) => fingers(markup).filter((match) => /class="[^"]*\bactive\b/.test(match[1]));

test('the rendered hand guide selects the physical finger across layouts', () => {
  for (const [key, layout, russian, code] of [
    ['a', 'qwerty', false, 'LP'], ['s', 'qwerty', false, 'LR'],
    ['d', 'qwerty', false, 'LM'], ['f', 'qwerty', false, 'LI'],
    ['j', 'qwerty', false, 'RI'], ['k', 'qwerty', false, 'RM'],
    ['l', 'qwerty', false, 'RR'], [';', 'qwerty', false, 'RP'],
    [' ', 'qwerty', false, 'RT'], ['s', 'dvorak', false, 'RP'],
    ['ф', 'dvorak', true, 'LP'], ['л', 'dvorak', true, 'RM'],
  ]) {
    const selected = active(html({ nextChar: key, keyboardLayout: layout, isRussian: russian }));
    assert.equal(selected.length, 1, `${layout}/${key}: exactly one finger`);
    assert.equal(selected[0][2], code, `${layout}/${key}`);
  }
});

test('each highlight includes the complete finger outline instead of a clipped band', () => {
  const all = fingers(html({ nextChar: 'f' }));
  assert.equal(all.length, 10);
  for (const finger of all) {
    const geometry = HAND_FINGERS.find((shape) => shape.code === finger[2][1]);
    const fill = finger[3].match(/<path\b[^>]*class="[^"]*\bfinger-fill\b[^>]*\bd="([^"]+)"/);
    assert.ok(fill, `missing full-finger fill: ${finger[2]}`);
    assert.equal(fill[1], `M${geometry.start}${geometry.contour}${geometry.base}`);
    assert.ok(HAND_OUTLINE.includes(geometry.contour), 'fill and visible silhouette share the outer contour');
  }
});

test('empty and unsupported keys leave both hands inactive', () => {
  for (const nextChar of ['', '€', 'Escape']) assert.equal(active(html({ nextChar })).length, 0);
});

test('multiple guides have no colliding SVG identifiers', () => {
  const ids = [...(html({}) + html({})).matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
  assert.equal(new Set(ids).size, ids.length);
});

test('Russian hand labels describe the active finger and Space key', () => {
  const markup = html({ nextChar: ' ', uiLang: 'ru' });
  assert.match(markup, /Правая рука · Большой/);
  assert.match(markup, /Пробел/);
  assert.match(html({ nextChar: 's', uiLang: 'ru' }), /Левая рука · Безымянный/);
});
