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
const transpile = source => ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText;
const ipcUrl = asModule(`
  let handlers;
  export const calls = [];
  export function setHandlers(value) { handlers = value; calls.length = 0; }
  export function ipcErrorMessage(error) { return String(error); }
  export async function getCustomTexts(limit) { calls.push(['texts',limit]); return handlers.texts(); }
  export async function searchCustomTexts(query,limit) { calls.push(['search',query,limit]); return handlers.search(query); }
  export async function getStatsHistory(limit,offset) { calls.push(['history',limit,offset]); return handlers.history(); }
  export async function getCourse(language) { calls.push(['course', language]); return handlers.course(language); }
  export async function getLessonProgress(language) { calls.push(['progress', language]); return handlers.progress(language); }
`);
const ipc = await import(ipcUrl);
const navigationUrl = asModule(transpile(readFileSync(new URL('../frontend/src/lib/lessonNavigation.ts', import.meta.url), 'utf8')));
const compiled = compileModule(transpile(readFileSync(new URL('../frontend/src/lib/stores/content.svelte.ts', import.meta.url), 'utf8')), {
  filename: 'content.svelte.js', dev: false,
}).js.code
  .replace("'svelte/internal/client'", JSON.stringify(pathToFileURL(require.resolve('svelte/internal/client')).href))
  .replace("'../api/ipc'", JSON.stringify(ipcUrl))
  .replace("'../lessonNavigation'", JSON.stringify(navigationUrl));
const { createContentStore } = await import(asModule(compiled));
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
const course = language => ({ modules: [{ id: language, lessons: [] }] });
const progress = language => [{ lesson_id: language, status: 'completed', best_wpm: 30, best_accuracy: 95 }];

test('slow old lesson loads cannot mix course/progress languages or overwrite a newer selection', async () => {
  const old = deferred();
  ipc.setHandlers({ course: language => language === 'en' ? old.promise : course(language), progress });
  const store = createContentStore({ setError() {} });
  const english = store.loadLessons();
  store.lessonLang = 'ru';
  await store.loadLessons();
  old.resolve(course('en'));
  await english;
  assert.equal(store.courseModules[0].id, 'ru');
  assert.deepEqual(Object.keys(store.lessonProgress), ['ru']);
  assert.deepEqual(ipc.calls, [['course', 'en'], ['course', 'ru'], ['progress', 'ru']]);
});

test('switching lesson language removes old clickable lessons until its course is ready', async () => {
  const russian = deferred();
  ipc.setHandlers({ course: language => language === 'ru' ? russian.promise : course(language), progress });
  const store = createContentStore({ setError() {} });
  await store.loadLessons();
  store.lessonLang = 'ru';
  const loading = store.loadLessons();
  assert.deepEqual(store.courseModules, []);
  assert.deepEqual(Object.keys(store.lessonProgress), []);
  russian.resolve(course('ru'));
  await loading;
  assert.equal(store.courseModules[0].id, 'ru');
});

test('an obsolete lesson-load error does not replace feedback from the current selection', async () => {
  const old = deferred();
  const errors = [];
  ipc.setHandlers({ course: language => language === 'en' ? old.promise : course(language), progress });
  const store = createContentStore({ setError: message => errors.push(message) });
  const english = store.loadLessons();
  store.lessonLang = 'ru';
  await store.loadLessons();
  old.reject(new Error('obsolete English failure'));
  await english;
  assert.deepEqual(errors, []);
  assert.equal(store.courseModules[0].id, 'ru');
});


test('a late custom-text search cannot overwrite a newer query', async () => {
  const old=deferred();
  ipc.setHandlers({search:q=>q==='old'?old.promise:[{id:2,name:'new'}]});
  const store=createContentStore({setError(){}});
  const loading=store.searchCustom('old');
  await store.searchCustom('new');
  old.resolve([{id:1,name:'old'}]); await loading;
  assert.equal(store.searchText,'new');
  assert.equal(store.customTexts[0].id,2);
});

test('clearing a search invalidates both pending search and earlier initial-list responses', async () => {
  const oldList=deferred(), oldSearch=deferred(); let reads=0;
  ipc.setHandlers({texts:()=>++reads===1?oldList.promise:[{id:3}],search:()=>oldSearch.promise});
  const store=createContentStore({setError(){}});
  const listing=store.loadCustomTexts(); const searching=store.searchCustom('old');
  await store.searchCustom('');
  oldSearch.resolve([{id:2}]); oldList.resolve([{id:1}]); await Promise.all([listing,searching]);
  assert.equal(store.searchText,''); assert.deepEqual(store.customTexts.map(t=>t.id),[3]);
});

test('an obsolete search failure neither rejects nor replaces current feedback/results', async () => {
  const old=deferred(), errors=[];
  ipc.setHandlers({search:q=>q==='old'?old.promise:[{id:2}]});
  const store=createContentStore({setError:m=>errors.push(m)});
  const searching=store.searchCustom('old'); await store.searchCustom('new');
  old.reject(new Error('obsolete')); await searching;
  assert.deepEqual(errors,[]); assert.equal(store.customTexts[0].id,2);
});

test('history badge initialization reads the persisted total without loading every row', async () => {
  ipc.setHandlers({history:()=>({total:7,tests:[]})});
  const store=createContentStore({setError(){}});
  await store.loadHistoryTotal();
  assert.equal(store.historyTotal,7); assert.deepEqual(ipc.calls,[['history',1,0]]);
});
