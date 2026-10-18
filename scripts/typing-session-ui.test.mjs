// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

// Compile the actual rune store with the existing frontend toolchain.
const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const { transpileModule, ScriptTarget, ModuleKind } = require('typescript');
const { compileModule } = require('svelte/compiler');
const asModule = (source) => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
const ipcUrl = asModule(`
  export const calls = [];
  export let nextOutput;
  export const starts = [];
  export const aborts = [];
  export const customStarts = [], lessonStarts = [], trainingLanguages = [];
  let processHandler, startHandler, abortHandler;
  export function setHandlers(handlers = {}) {
    processHandler = handlers.process;
    startHandler = handlers.start;
    abortHandler = handlers.abort;
  }
  export async function startTest(params) {
    starts.push(params);
    if (startHandler) return startHandler(params);
    return {session_id:'restart-'+starts.length,text:params.text ?? 'привет',text_length:6,mode_type:params.mode ?? 'time',mode_config:{duration:30,language:params.language},language:params.language};
  }
  export async function abortSession(id) { aborts.push(id); return abortHandler?.(id); }
  export async function generateWeakKeysTraining(language) { trainingLanguages.push(language); return 'weak keys'; }
  export async function startCustomTextTest(id) {
    customStarts.push(id);
    return {session_id:'custom-'+customStarts.length,text:'фыва',text_length:4,mode_type:'custom',mode_config:{language:'ru'},language:'ru'};
  }
  export async function startLesson(id, language) {
    lessonStarts.push([id,language]);
    return {session_id:'lesson-'+lessonStarts.length,text:'фыва',text_length:4,mode_type:'lesson',mode_config:{language},language};
  }
  export function ipcErrorMessage(error) { return String(error); }

  export function setOutput(value) { nextOutput = value; }
  export async function processKey(...args) { calls.push(args); return processHandler ? processHandler(...args) : nextOutput; }
`);
const ipc = await import(ipcUrl);
const source = readFileSync(new URL('../frontend/src/lib/stores/testSession.svelte.ts', import.meta.url), 'utf8');
const js = transpileModule(source, {
  compilerOptions: { target: ScriptTarget.ESNext, module: ModuleKind.ESNext },
}).outputText;
const compiled = compileModule(js, { filename: 'testSession.svelte.js', dev: false }).js.code
  .replace("'svelte/internal/client'", JSON.stringify(pathToFileURL(require.resolve('svelte/internal/client')).href))
  .replace("'../api/ipc'", JSON.stringify(ipcUrl))
  .replace("'../i18n'", JSON.stringify(asModule('export const t = (_, key) => key;')));
const { createTestSessionStore } = await import(asModule(compiled));

function store(overrides = {}) {
  return createTestSessionStore({
    getSettings: () => null, getUiLang: () => 'en', setError() {}, setZenActive() {},
    playSound() {}, notify() {}, beforeStart: async () => {}, getCurrentLessonId: () => null,
    setCurrentLessonId() {}, onLessonCompleted() {}, onHistoryChanged() {},
    onAchievementsChanged: async () => {}, onStarted() {}, ...overrides,
  });
}
const response = {
  session_id: 'fixture', text: 'hello', text_length: 5, mode_type: 'time',
  mode_config: { duration: 30, language: 'en' }, language: 'en',
};

test('time session uses the untagged duration actually emitted by Rust', () => {
  const session = store();
  session.startTestFromResponse(response);
  assert.equal(session.sessionDurationMs, 30_000);
  session.startTestFromResponse({ ...response, mode_type: 'words', mode_config: { word_count: 25, language: 'en' } });
  assert.equal(session.sessionDurationMs, 0);
});

test('backend-confirmed completion stops the session and freezes final elapsed time', async () => {
  const session = store();
  session.startTestFromResponse(response);
  ipc.setOutput({
    session_state: 'persisted', caret_pos: 5, key_result: 'test_ended', live_stats: null,
    test_complete: { duration_ms: 30_000, accuracy: 90, wpm: 40 },
  });
  session.enqueueKey('', '', true);
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(session.isRunning, false);
  assert.equal(session.isComplete, true);
  assert.equal(session.elapsedMs, 30_000);
  assert.equal(session.testStartedAt, null);
});

test('native input sends the committed Cyrillic character to IPC exactly once', async () => {
  const session = store();
  session.startTestFromResponse({ ...response, language: 'ru' });
  ipc.calls.length = 0;
  ipc.setOutput({ session_state: 'running', caret_pos: 0, key_result: 'noop', live_stats: null, test_complete: null });
  session.acceptTypingKey('ф', 'KeyA');
  await new Promise(resolve => setImmediate(resolve));
  assert.deepEqual(ipc.calls, [['ф', 'KeyA', 'fixture']]);
  assert.equal(session.lastTypedChar, 'ф');
  assert.notEqual(session.testStartedAt, null);
});

test('restarting five completed RU tests clears layout evidence and retains RU', async () => {
  const session = store();
  session.selectedLanguage = 'ru';
  await session.startTest();
  for (let repeat = 0; repeat < 5; repeat++) {
    session.lastTypedChar = 'f';
    session.capsLockOn = true;
    ipc.setOutput({ session_state: 'persisted', caret_pos: 6, key_result: 'test_ended', live_stats: null,
      test_complete: { duration_ms: 30_000, accuracy: 90, wpm: 40 } });
    session.enqueueKey('', '', true);
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(session.isComplete, true);
    await session.restartTest();
    assert.equal(session.sessionLanguage, 'ru');
    assert.equal(session.lastTypedChar, '');
    assert.equal(session.capsLockOn, false);
    assert.equal(ipc.starts.at(-1).language, 'ru');
  }
});

test('input without a running session cannot set layout evidence or call IPC', () => {
  const session = store();
  ipc.calls.length = 0;
  session.acceptTypingKey('a', 'KeyA');
  assert.equal(session.lastTypedChar, '');
  assert.deepEqual(ipc.calls, []);
});

const tick = () => new Promise(resolve => setImmediate(resolve));
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
const runningOutput = { session_state: 'running', caret_pos: 0, key_result: 'noop', live_stats: null, test_complete: null };

test('a rejected key from an aborted session cannot discard the new session queue', async () => {
  const oldKey = deferred();
  const errors = [];
  ipc.calls.length = 0;
  ipc.setHandlers({ process: (_, __, id) => id === 'old' ? oldKey.promise : runningOutput });
  try {
    const session = store({ setError: message => errors.push(message) });
    session.startTestFromResponse({ ...response, session_id: 'old' });
    session.acceptTypingKey('h', 'KeyH');
    await session.restartTest();
    session.acceptTypingKey('п', 'KeyG');
    oldKey.reject(new Error('session no longer active'));
    await tick();
    assert.deepEqual(ipc.calls.map(call => call[0]), ['h', 'п']);
    assert.equal(errors.some(message => message.startsWith('Typing error:')), false);
    assert.equal(session.isRunning, true);
  } finally { ipc.setHandlers(); }
});

test('a second restart cannot abort or clear the replacement session', async () => {
  const aborts = [deferred(), deferred()];
  ipc.aborts.length = 0;
  ipc.setHandlers({ abort: () => aborts[ipc.aborts.length - 1].promise });
  try {
    const session = store();
    session.startTestFromResponse(response);
    const first = session.restartTest();
    const second = session.restartTest();
    assert.equal(ipc.aborts.length, 1);
    aborts[0].resolve();
    await first;
    const replacementText = session.text;
    aborts[1].resolve();
    await second;
    assert.equal(session.isRunning, true);
    assert.equal(session.text, replacementText);
    assert.equal(ipc.aborts.length, 1);
  } finally { ipc.setHandlers(); }
});

test('a configuration click during a pending start cannot diverge from the accepted request', async () => {
  const start = deferred();
  ipc.setHandlers({ start: () => start.promise });
  try {
    const session = store();
    const starting = session.startTest();
    await tick();
    session.onLanguageChange('ru');
    await tick();
    start.resolve(response);
    await starting;
    assert.equal(session.selectedLanguage, session.sessionLanguage);
    assert.equal(session.selectedLanguage, 'en');
  } finally { ipc.setHandlers(); }
});

test('abort and restart share one pending abort and cannot clear the replacement later', async () => {
  const abort = deferred();
  ipc.aborts.length = 0;
  ipc.setHandlers({ abort: () => abort.promise });
  try {
    const session = store();
    session.startTestFromResponse(response);
    const stopping = session.abortTest();
    const restarting = session.restartTest();
    assert.equal(ipc.aborts.length, 1);
    abort.resolve();
    await Promise.all([stopping, restarting]);
    assert.equal(session.isRunning, true);
    assert.equal(session.text, 'привет');
  } finally { ipc.setHandlers(); }
});

test('dashboard training navigates only after a successful backend start; inline training stays inline', async () => {
  let navigations = 0;
  const session = store({ onStarted: () => { navigations++; } });
  await session.startTraining(true);
  assert.equal(navigations, 1);
  assert.equal(session.isRunning, true);
  assert.equal(ipc.starts.at(-1).mode, 'custom');
  await session.abortTest();
  await session.startTraining();
  assert.equal(navigations, 1);
  await session.abortTest();
  ipc.setHandlers({ start: () => { throw new Error('start refused'); } });
  try {
    await session.startTraining(true);
    assert.equal(navigations, 1);
    assert.equal(session.isRunning, false);
  } finally { ipc.setHandlers(); }
});


test('restart retains the Russian custom text ID instead of starting the global English timer', async () => {
  const session = store();
  ipc.customStarts.length = 0;
  const startsBefore = ipc.starts.length;
  await session.startCustomTest(42);
  assert.equal(session.selectedLanguage, 'ru');
  ipc.setOutput({session_state:'persisted',caret_pos:4,key_result:'test_ended',live_stats:null,
    test_complete:{duration_ms:1000,accuracy:100,wpm:48}});
  session.enqueueKey('', '', true);
  await tick();
  await session.restartTest();
  assert.deepEqual(ipc.customStarts, [42,42]);
  assert.equal(ipc.starts.length, startsBefore);
  assert.equal(session.sessionModeType, 'custom');
  assert.equal(session.sessionLanguage, 'ru');
});

test('restart retains lesson identity and language while aborting its previous running identity', async () => {
  const session = store();
  ipc.lessonStarts.length = 0;
  await session.startLesson('home-row-ru','ru');
  const previous = 'lesson-'+ipc.lessonStarts.length;
  await session.restartTest();
  assert.deepEqual(ipc.lessonStarts,[['home-row-ru','ru'],['home-row-ru','ru']]);
  assert.equal(ipc.aborts.at(-1),previous);
  assert.equal(session.sessionModeType,'lesson');
});

test('weak-key restart retains its training mode and language', async () => {
  const session = store();
  session.selectedLanguage = 'ru';
  ipc.trainingLanguages.length = 0;
  await session.startTraining();
  await session.restartTest();
  assert.deepEqual(ipc.trainingLanguages,['ru','ru']);
  assert.equal(ipc.starts.at(-1).mode,'custom');
  assert.equal(session.sessionLanguage,'ru');
});

test('explicit mode selection replaces custom repeat context only after a successful start', async () => {
  const session = store();
  await session.startCustomTest(42);
  session.onModeChange('words');
  await tick();
  await session.restartTest();
  assert.equal(ipc.starts.at(-1).mode,'words');
  assert.equal(session.sessionModeType,'words');
});

test('a rejected explicit mode start does not discard the accepted custom repeat source', async () => {
  const session=store();ipc.customStarts.length=0;
  await session.startCustomTest(42);
  ipc.setHandlers({start:()=>{throw new Error('replacement refused');}});
  try {
    session.onModeChange('words');await tick();
    await session.restartTest();
    assert.deepEqual(ipc.customStarts,[42,42]);
    assert.equal(session.sessionModeType,'custom');
  } finally {ipc.setHandlers();}
});
