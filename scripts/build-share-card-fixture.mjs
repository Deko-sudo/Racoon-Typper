#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { createRequire } from 'node:module';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

if (!process.argv[2]) throw new Error('Usage: node scripts/build-share-card-fixture.mjs OUTPUT_DIR');
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(resolve(root, 'frontend/package.json'));
const { build } = await import(pathToFileURL(require.resolve('vite')).href);
await build({
  configFile: false,
  root,
  build: {
    outDir: resolve(process.argv[2]),
    emptyOutDir: false,
    rollupOptions: { input: resolve(root, 'scripts/fixtures/share-card.html') },
  },
});
