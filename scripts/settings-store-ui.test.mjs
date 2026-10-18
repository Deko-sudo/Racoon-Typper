// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const ts = require('typescript');
const { compileModule } = require('svelte/compiler');
const asModule = source => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
const ipcUrl = asModule(`
  let handlers;
  export const writes=[];
  export function configure(value) {handlers=value; writes.length=0;}
  export async function getThemeCss(name) {return handlers.css(name);}
  export async function getSettings() {return handlers.settings();}
  export async function setSetting(key,value) {writes.push(value); return handlers.save(value);}
  export async function getThemes() {return [];}
  export function ipcErrorMessage(error) {return error.message ?? String(error);}
`);
const ipc = await import(ipcUrl);
const js = ts.transpileModule(readFileSync(new URL('../frontend/src/lib/stores/settings.svelte.ts', import.meta.url), 'utf8'), {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText;
const compiled = compileModule(js, { filename: 'settings.svelte.js', dev: false }).js.code
  .replace("'svelte/internal/client'", JSON.stringify(pathToFileURL(require.resolve('svelte/internal/client')).href))
  .replace("'../api/ipc'", JSON.stringify(ipcUrl));
const { createSettingsStore } = await import(asModule(compiled));
const tick = () => new Promise(resolve => setImmediate(resolve));
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => {resolve=yes; reject=no;});
  return {promise, resolve, reject};
}
function setup(handlers={}) {
  const values=new Map(), nodes=new Map(), errors=[];
  globalThis.document={
    documentElement:{dataset:{},style:{setProperty:(k,v)=>values.set(k,v), removeProperty:k=>values.delete(k)}},
    getElementById:id=>nodes.get(id), createElement:()=>({setAttribute(){},textContent:''}),
    head:{appendChild:node=>nodes.set(node.id,node)},
  };
  let saved={theme:'racoon_graphite',ui_language:'en'};
  ipc.configure({css:name=>`:root { --color-accent: ${name}; }`,settings:()=>saved,
    save:name=>(saved={...saved,theme:name}),...handlers});
  return {store:createSettingsStore({setError:m=>errors.push(m)}),values,errors};
}

test('late theme CSS cannot overwrite the selected, rendered or persisted newer theme', async () => {
  const old=deferred();
  const {store,values}=setup({css:name=>name==='old'?old.promise:':root { --color-accent: new; }'});
  const first=store.selectTheme('old');
  await store.selectTheme('new');
  old.resolve(':root { --color-accent: old; }'); await first;
  assert.equal(document.documentElement.dataset.theme,'new');
  assert.equal(values.get('--color-accent'),'new');
  assert.equal(store.activeTheme,'new');
  assert.equal(store.settings.theme,'new');
  assert.deepEqual(ipc.writes,['new']);
});

test('an already-started theme save is serialized before the newer save', async () => {
  const oldSave=deferred(); let saved='racoon_graphite';
  const {store}=setup({settings:()=>({theme:saved}),save:async name=>{
    if(name==='old') await oldSave.promise;
    saved=name; return {theme:name};
  }});
  const first=store.selectTheme('old'); await tick();
  const second=store.selectTheme('new'); await tick();
  assert.deepEqual(ipc.writes,['old']);
  oldSave.resolve(); await Promise.all([first,second]);
  assert.deepEqual(ipc.writes,['old','new']);
  assert.equal(saved,'new'); assert.equal(store.activeTheme,'new');
  assert.equal(document.documentElement.dataset.theme,'new');
});

test('an obsolete theme failure cannot replace newer success feedback', async () => {
  const old=deferred();
  const {store,errors}=setup({css:name=>name==='old'?old.promise:':root { --color-accent: new; }'});
  const first=store.selectTheme('old'); await store.selectTheme('new');
  old.reject(new Error('obsolete failure')); await first;
  assert.equal(store.activeTheme,'new');
  assert.equal(errors.some(m=>m.startsWith('Theme error:')),false);
});

test('a current theme save failure restores the persisted theme instead of leaving an unsaved preview', async () => {
  const {store,errors}=setup({save:()=>{throw new Error('save failed');}});
  await store.loadSettings(); await store.selectTheme('new');
  assert.equal(store.activeTheme,'racoon_graphite');
  assert.equal(document.documentElement.dataset.theme,'racoon_graphite');
  assert.equal(errors.some(m=>m.startsWith('Theme error:')),true);
});

test('failed newer CSS waits for an earlier in-flight save before reconciling the rendered theme', async () => {
  const oldSave=deferred(); let saved='racoon_graphite', reads=0;
  const {store}=setup({css:name=>{
    if(name==='new') throw new Error('new CSS unavailable');
    return `:root { --color-accent: ${name}; }`;
  },settings:()=>{reads++;return {theme:saved};},save:async name=>{
    await oldSave.promise; saved=name;return {theme:name};
  }});
  await store.loadSettings();
  const first=store.selectTheme('old');await tick();
  const second=store.selectTheme('new');await tick();
  assert.equal(reads,1);
  oldSave.resolve();await Promise.all([first,second]);
  assert.equal(saved,'old');assert.equal(store.activeTheme,'old');
  assert.equal(document.documentElement.dataset.theme,'old');
});
