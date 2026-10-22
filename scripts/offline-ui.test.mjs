// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { i18nUrl } from './i18n-test-loader.mjs';

const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const { compile } = require('svelte/compiler');
const ts = require('typescript');
const { render } = await import(pathToFileURL(require.resolve('svelte/server')).href);
const moduleUrl = source => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
const compileComponent = (source, name) => compile(source, { filename: name, generate: 'server', dev: false }).js.code
  .replace("'svelte/internal/server'", JSON.stringify(pathToFileURL(require.resolve('svelte/internal/server')).href));
const emptyChild = moduleUrl('export default () => {};');
const mocks = moduleUrl('export const open = () => {}; export const readTextFile = () => {}; export const readText = () => {};');
const updaterMocks = moduleUrl('export const checkForUpdate = () => {}; export const installUpdate = () => {};');

async function view(name) {
  const source = readFileSync(new URL(`../frontend/src/components/${name}.svelte`, import.meta.url), 'utf8');
  const compiled = compileComponent(source, `${name}.svelte`)
    .replaceAll(/'\.\/[^']+\.svelte'/g, JSON.stringify(emptyChild))
    .replace("'../lib/i18n'", JSON.stringify(i18nUrl))
    .replace("'../lib/updater'", JSON.stringify(updaterMocks))
    .replaceAll(/'@tauri-apps\/plugin-[^']+'/g, JSON.stringify(mocks))
    .replace("'../lib/api/ipc'", JSON.stringify(moduleUrl('export {};')))
    .replace("'../lib/textPack'", JSON.stringify(moduleUrl(ts.transpileModule(
      readFileSync(new URL('../frontend/src/lib/textPack.ts', import.meta.url), 'utf8'), {
        compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
      }).outputText)));
  return (await import(moduleUrl(compiled))).default;
}

function sources(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const url = new URL(entry.name + (entry.isDirectory() ? '/' : ''), directory);
    return entry.isDirectory() ? sources(url) : /\.(svelte|ts)$/.test(entry.name) ? [[url, readFileSync(url, 'utf8')]] : [];
  });
}

test('frontend has no HTTP client, URL-import command, or updater runtime path', () => {
  for (const [file, source] of sources(new URL('../frontend/src/', import.meta.url))) {
    assert.doesNotMatch(source, /\bfetch\s*\(|\b(?:WebSocket|EventSource|XMLHttpRequest)\b|@tauri-apps\/plugin-updater|import_text_from_url|importTextFromUrl/, file.pathname);
  }
  const manifest = JSON.parse(readFileSync(new URL('../frontend/package.json', import.meta.url), 'utf8'));
  const lock = JSON.parse(readFileSync(new URL('../frontend/package-lock.json', import.meta.url), 'utf8'));
  assert.equal(manifest.dependencies['@tauri-apps/plugin-updater'], undefined);
  assert.equal(lock.packages[''].dependencies['@tauri-apps/plugin-updater'], undefined);
  assert.equal(lock.packages['node_modules/@tauri-apps/plugin-updater'], undefined);
});

test('Settings exposes local maintenance without update download/install controls', async () => {
  const SettingsView = await view('SettingsView');
  const markup = render(SettingsView, { props: { settings: { practice_languages: ['en'] }, themes: [], activeTheme: 'racoon_graphite' } }).body;
  assert.doesNotMatch(markup, /Check for updates|Install v|<h3>Updates<\/h3>/);
  assert.match(markup, /Clear all statistics/);
  assert.match(markup, /Restore database/);
});

test('CustomTexts keeps file/clipboard import and removes URL import', async () => {
  const CustomTextsView = await view('CustomTextsView');
  const markup = render(CustomTextsView, { props: {
    customTexts: [], searchText: '', showEditor: true, newName: '', newTextContent: '', newLanguage: 'en',
  } }).body;
  assert.doesNotMatch(markup, /Import URL|https:\/\//);
  assert.match(markup, /Import \.txt/);
  assert.match(markup, /Paste/);
});
