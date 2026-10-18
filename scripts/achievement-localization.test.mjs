// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { t, UI_LANGUAGES, achievementMessages, i18nUrl } from './i18n-test-loader.mjs';

const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const { compile } = require('svelte/compiler');
const ts = require('typescript');
const asModule = (source) => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
const helperJs = ts.transpileModule(readFileSync(new URL('../frontend/src/lib/achievements.ts', import.meta.url), 'utf8'), {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText.replace("'./i18n'", JSON.stringify(i18nUrl));
const helperUrl = asModule(helperJs);
const { localizeAchievement } = await import(helperUrl);

const backend = readFileSync(new URL('../crates/core/src/analytics.rs', import.meta.url), 'utf8');
const catalog = [...backend.slice(backend.indexOf('let achievements = vec!['), backend.indexOf('// Сортируем:'))
  .matchAll(/mk\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"/g)]
  .map(([, id, name, description]) => ({ id, name, description }));

test('all current backend achievements have names in every selectable UI language', () => {
  assert.equal(catalog.length, 52);
  for (const [lang] of UI_LANGUAGES) {
    for (const achievement of catalog) {
      const key = `achievement.${achievement.id}.name`;
      assert.ok(Object.hasOwn(achievementMessages[lang], key), `${lang}/${key}: no English fallback`);
      assert.notEqual(t(lang, key), key, `${lang}/${achievement.id}`);
    }
  }
  assert.equal(t('ru', 'achievement.first_test.name'), 'Первые шаги');
  assert.equal(t('de', 'achievement.first_test.name'), 'Erste Schritte');
  assert.equal(t('ja', 'achievement.first_test.name'), 'はじめの一歩');
});

test('gallery controls and empty/error states are translated with the UI', () => {
  for (const [lang] of UI_LANGUAGES) {
    for (const key of ['all', 'unlocked', 'locked', 'empty_category', 'load_error']) {
      assert.ok(Object.hasOwn(achievementMessages[lang], `achievements.${key}`), `${lang}/${key}`);
      assert.notEqual(t(lang, `achievements.${key}`), `achievements.${key}`, `${lang}/${key}`);
    }
  }
  assert.equal(t('ru', 'achievements.all'), 'Все');
  assert.equal(t('ru', 'achievements.unlocked'), 'Получено');
  assert.equal(t('ru', 'achievements.locked'), 'Не получено');
});

test('every localized goal preserves backend thresholds and fills all placeholders', () => {
  for (const achievement of catalog) {
    assert.deepEqual(localizeAchievement('en', achievement), { name: achievement.name, description: achievement.description });
    for (const [lang] of UI_LANGUAGES) {
      const copy = localizeAchievement(lang, achievement);
      assert.doesNotMatch(copy.description, /[{}]|achievement\.goal\./, `${lang}/${achievement.id}`);
      const digits = (value) => value.replace(/(?<=\d)[,\u00a0\u202f.](?=\d)/g, '').match(/\d+/g)?.sort() ?? [];
      assert.deepEqual(digits(copy.description), digits(achievement.description), `${lang}/${achievement.id}`);
      const templateKeys = Object.keys(achievementMessages.en).filter(key => key.startsWith('achievement.goal.'));
      for (const key of templateKeys) assert.ok(Object.hasOwn(achievementMessages[lang], key), `${lang}/${key}`);
    }
  }
  const russian = localizeAchievement('ru', catalog.find(a => a.id === 'balanced_50_95'));
  assert.equal(russian.name, 'Баланс');
  assert.equal(russian.description, 'Достигните скорости 50 WPM и точности 95% в одном тесте');
});

test('changing UI language translates existing data without mutating backend state', () => {
  const achievement = Object.freeze({ ...catalog[0], unlocked: true, unlocked_at: '2026-10-15T00:00:00Z' });
  assert.equal(localizeAchievement('ru', achievement).name, 'Первые шаги');
  assert.equal(localizeAchievement('de', achievement).name, 'Erste Schritte');
  assert.equal(localizeAchievement('en', achievement).name, 'First Steps');
  assert.equal(achievement.name, 'First Steps');
  assert.equal(achievement.unlocked, true);
});

test('small lesson counts and streaks use the locale plural form', () => {
  const lesson = catalog.find(a => a.id === 'lessons_2');
  assert.equal(localizeAchievement('ru', lesson).description, 'Пройдите 2 урока');
  assert.equal(localizeAchievement('uk', lesson).description, 'Пройдіть 2 уроки');
  assert.equal(localizeAchievement('pl', lesson).description, 'Ukończ 2 lekcje');
  assert.equal(localizeAchievement('cs', lesson).description, 'Dokončete 2 lekce');
  assert.equal(localizeAchievement('de', catalog.find(a => a.id === 'streak_1')).description, 'Serie: 1 Tag in Folge');
  assert.equal(localizeAchievement('ro', catalog.find(a => a.id === '5_tests')).description, 'Finalizează 5 teste');
  assert.equal(localizeAchievement('ro', catalog.find(a => a.id === '25_tests')).description, 'Finalizează 25 de teste');
});

test('unknown backend IDs keep their text and unknown UI languages fall back to English', () => {
  for (const id of ['future_badge', '__proto__', 'constructor']) {
    assert.deepEqual(localizeAchievement('ru', { id, name: 'Future badge', description: 'New goal' }), {
      name: 'Future badge', description: 'New goal',
    });
  }
  assert.equal(localizeAchievement('not-a-locale', catalog[0]).name, 'First Steps');
});

test('the actual gallery renders localized controls and empty states', async () => {
  const resolve = (name) => JSON.stringify(pathToFileURL(require.resolve(name)).href);
  const component = (name) => compile(readFileSync(new URL(`../frontend/src/components/${name}.svelte`, import.meta.url), 'utf8'), {
    filename: `${name}.svelte`, generate: 'server', dev: false,
  }).js.code.replace("'svelte/internal/server'", resolve('svelte/internal/server'));
  let gallery = component('AchievementGallery')
    .replace("'svelte'", resolve('svelte'))
    .replace("'../lib/api/ipc'", JSON.stringify(asModule('export async function getAchievements() { return []; }')))
    .replace("'../lib/i18n'", JSON.stringify(i18nUrl))
    .replace("'../lib/achievements'", JSON.stringify(helperUrl))
    .replace("'./Icon.svelte'", JSON.stringify(asModule(component('Icon'))));
  const { default: Gallery } = await import(asModule(gallery));
  const { render } = await import(pathToFileURL(require.resolve('svelte/server')).href);
  for (const [lang] of UI_LANGUAGES) {
    const markup = render(Gallery, { props: { uiLang: lang } }).body;
    for (const key of ['achievements.all', 'achievements.unlocked', 'achievements.locked', 'analytics.empty_achievements']) {
      assert.ok(markup.includes(t(lang, key)), `${lang}/${key}`);
    }
  }
});
