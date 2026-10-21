// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { i18nUrl } from './i18n-test-loader.mjs';

const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const ts = require('typescript');
const { compileModule } = require('svelte/compiler');
const asModule = source => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
const transpile = source => ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText;
const ipcUrl = asModule(`
  let preview;
  export const imports = [];
  export function setPreview(handler) { preview = handler; imports.length = 0; }
  export async function previewTextPackImport(...args) { return preview(...args); }
  export async function importTextPack(...args) { imports.push(args); return {to_insert:1,to_skip:0,removed_by_replace:0}; }
  export function ipcErrorMessage(error) { return error instanceof Error ? error.message : String(error); }
`);
const ipc = await import(ipcUrl);
// Exercise the real component's event handlers and derived state, compiled by
// Svelte. Only props/DOM file selection are supplied by a deterministic harness;
// the production handler bodies are not reimplemented here.
const componentSource = readFileSync(new URL('../frontend/src/components/CustomTextsView.svelte', import.meta.url), 'utf8');
const script = transpile(componentSource.match(/<script[^>]*>([\s\S]*?)<\/script>/)[1]);
const program = ts.createSourceFile('CustomTextsView.js', script, ts.ScriptTarget.ESNext, true, ts.ScriptKind.JS);
const imports = program.statements.filter(ts.isImportDeclaration).map(statement => statement.getText(program)).join('\n');
let body = program.statements.filter(statement => !ts.isImportDeclaration(statement)).map(statement => statement.getText(program)).join('\n');
body = body.replace('$props()', 'props');
const harness = `${imports}\nexport function createTextPackHarness(props) {${body}
  return {
    choosePackFile, previewPack, applyPack,
    get document() { return packDocument; },
    get plan() { return packPreview; },
    get canApply() { return canApplyPack; },
    get error() { return packError; },
    get acknowledged() { return packReplaceAck; },
    set acknowledged(value) { packReplaceAck = value; },
    set policy(value) { packPolicy = value; packPreview = null; },
  };
}`;
const mocks = asModule('export const open = () => {}; export const readTextFile = () => {}; export const readText = () => {};');
const compiled = compileModule(harness, { filename: 'CustomTextsView.svelte.js', dev: false }).js.code
  .replace("'svelte/internal/client'", JSON.stringify(pathToFileURL(require.resolve('svelte/internal/client')).href))
  .replaceAll(/'@tauri-apps\/plugin-[^']+'/g, JSON.stringify(mocks))
  .replace("'../lib/api/ipc'", JSON.stringify(ipcUrl))
  .replace("'../lib/i18n'", JSON.stringify(i18nUrl))
  .replace("'../lib/textPack'", JSON.stringify(asModule(transpile(readFileSync(new URL('../frontend/src/lib/textPack.ts', import.meta.url), 'utf8')))));
const { createTextPackHarness } = await import(asModule(compiled));
const pack = () => createTextPackHarness({ customTexts: [], searchText: '', newLanguage: 'en', uiLang: 'en', onSearch() {} });
const fileEvent = (name, contents, size = Buffer.byteLength(contents)) => ({ currentTarget: {
  files: [{ name, size, text: async () => contents }], value: name,
} });
const plan = { policy: 'merge', language: 'en', incoming: 1, to_insert: 1, to_skip: 0, removed_by_replace: 0 };
function deferred() {
  let resolve;
  const promise = new Promise(yes => { resolve = yes; });
  return { promise, resolve };
}

test('an old preview cannot authorize importing a subsequently selected document', async () => {
  const pending = deferred();
  ipc.setPreview(() => pending.promise);
  const state = pack();
  await state.choosePackFile(fileEvent('first.json', 'first'));
  const previewing = state.previewPack();
  await state.choosePackFile(fileEvent('second.csv', 'second'));
  pending.resolve(plan);
  await previewing;
  assert.equal(state.canApply, false);
  await state.applyPack();
  assert.deepEqual(ipc.imports, []);
});

test('a preview binds document, source format and policy and resets replace acknowledgement', async () => {
  ipc.setPreview(async () => plan);
  const state = pack();
  state.policy = 'replace';
  state.acknowledged = true;
  await state.choosePackFile(fileEvent('first.json', 'same document'));
  assert.equal(state.acknowledged, false);
  await state.previewPack();
  assert.equal(state.canApply, false);
  state.acknowledged = true;
  assert.equal(state.canApply, true);
  await state.applyPack();
  assert.deepEqual(ipc.imports, [['same document', 'json', 'replace']]);
});

test('preview of one policy cannot authorize another policy after an async response', async () => {
  const pending = deferred();
  ipc.setPreview(() => pending.promise);
  const state = pack();
  await state.choosePackFile(fileEvent('first.json', 'document'));
  const previewing = state.previewPack();
  state.policy = 'replace';
  state.acknowledged = true;
  pending.resolve(plan);
  await previewing;
  assert.equal(state.canApply, false);
  await state.applyPack();
  assert.deepEqual(ipc.imports, []);
});

test('empty and oversized text-pack files are rejected before allocating their contents', async () => {
  for (const size of [0, 4 * 1024 * 1024 + 1]) {
    let reads = 0;
    const event = fileEvent('large.json', 'x', size);
    event.currentTarget.files[0].text = async () => { reads++; return 'x'; };
    const state = pack();
    await state.choosePackFile(event);
    assert.equal(reads, 0);
    assert.equal(state.document, '');
    assert.notEqual(state.error, '');
  }
});

test('a slow previous file read cannot overwrite the latest selected pack', async () => {
  const oldFile = deferred();
  const first = fileEvent('old.json', 'old');
  first.currentTarget.files[0].text = () => oldFile.promise;
  const state = pack();
  const reading = state.choosePackFile(first);
  await state.choosePackFile(fileEvent('new.csv', 'new'));
  oldFile.resolve('old');
  await reading;
  assert.equal(state.document, 'new');
});

test('same contents selected in a different source format require a new preview', async () => {
  const pending = deferred();
  ipc.setPreview(() => pending.promise);
  const state = pack();
  await state.choosePackFile(fileEvent('first.json', 'same contents'));
  const previewing = state.previewPack();
  await state.choosePackFile(fileEvent('second.csv', 'same contents'));
  pending.resolve(plan);
  await previewing;
  assert.equal(state.canApply, false);
});

test('the boundary-sized file is read and a failed next read invalidates its preview', async () => {
  ipc.setPreview(async () => plan);
  const state = pack();
  await state.choosePackFile(fileEvent('boundary.json', 'valid', 4 * 1024 * 1024));
  await state.previewPack();
  assert.equal(state.canApply, true);
  const broken = fileEvent('broken.json', 'unused');
  broken.currentTarget.files[0].text = async () => { throw new Error('read failed'); };
  await state.choosePackFile(broken);
  assert.equal(state.canApply, false);
  assert.equal(state.document, '');
  assert.equal(state.error, 'read failed');
});
