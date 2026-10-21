// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

// Compile the real dictionary modules with the project's existing TypeScript.
// Data URLs keep tests independent of Node's extensionless TS import handling.
const require = createRequire(new URL('../frontend/package.json', import.meta.url));
const ts = require('typescript');
const moduleUrl = (source) => `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`;
const compile = (name) => ts.transpileModule(readFileSync(new URL(`../frontend/src/lib/${name}.ts`, import.meta.url), 'utf8'), {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText;
const messagesUrl = moduleUrl(compile('achievementMessages'));
export const i18nUrl = moduleUrl(compile('i18n').replace("'./achievementMessages'", JSON.stringify(messagesUrl)));
export const { t, UI_LANGUAGES } = await import(i18nUrl);
export const { achievementMessages } = await import(messagesUrl);
