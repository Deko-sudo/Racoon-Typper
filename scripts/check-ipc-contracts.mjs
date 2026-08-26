#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const ipc = readFileSync(resolve(root, 'frontend/src/lib/api/ipc.ts'), 'utf8');
const main = readFileSync(resolve(root, 'crates/app/src/main.rs'), 'utf8');
const clientCommands = new Set([...ipc.matchAll(/invoke[^(]*\('([a-z_]+)'/g)].map((match) => match[1]));
const registeredCommands = new Set([...main.matchAll(/commands::\w+::([a-z_]+)/g)].map((match) => match[1]));

const missingFromBackend = [...clientCommands].filter((command) => !registeredCommands.has(command));
const missingFromClient = [...registeredCommands].filter((command) => !clientCommands.has(command));
if (missingFromBackend.length || missingFromClient.length) {
  console.error('IPC contract mismatch:');
  if (missingFromBackend.length) console.error(`  client-only: ${missingFromBackend.join(', ')}`);
  if (missingFromClient.length) console.error(`  backend-only: ${missingFromClient.join(', ')}`);
  process.exit(1);
}
console.log(`IPC contract check passed (${clientCommands.size} commands)`);
