#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyVendor, checkGlibAdvisories, checkMacroAdvisories } from './vendor-provenance.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const record = JSON.parse(readFileSync(resolve(root, 'licenses/vendor-glib.json')));
verifyVendor(root, record);
const macros = ['glib-macros', 'gtk3-macros'].map(name =>
  JSON.parse(readFileSync(resolve(root, `licenses/vendor-${name}.json`))));
for (const macro of macros) verifyVendor(root, macro);
const reference = resolve(root, 'scripts/glib-advisory-reference/Cargo.toml');
const offline = process.env.CARGO_NET_OFFLINE === 'true' ? ['--offline'] : [];
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--manifest-path', reference, '--locked', ...offline, '--format-version', '1'], { encoding: 'utf8' }));
const glib = metadata.packages.filter(pkg => pkg.name === 'glib');
if (glib.length !== 1 || glib[0].version !== record.version || glib[0].source !== record.upstream_source) {
  throw new Error('GLib advisory reference must use the pinned crates.io package');
}
for (const macro of macros) {
  const matches = metadata.packages.filter(pkg => pkg.name === macro.name);
  if (matches.length !== 1 || matches[0].version !== macro.version || matches[0].source !== macro.upstream_source) {
    throw new Error(`Macro advisory reference must use the pinned crates.io package: ${macro.name}`);
  }
}
let output;
try {
  output = execFileSync('cargo-deny', ['--manifest-path', reference, '--config', resolve(root, 'deny.toml'), '--locked', ...offline,
    '--format', 'json', 'check', 'advisories', '-D', 'unmaintained', '-D', 'unsound'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
} catch (error) {
  if (error.status !== 1) throw error;
  output = (error.stdout || '') + (error.stderr || '');
}
const events = output.split('\n').filter(Boolean).map(line => JSON.parse(line));
// Preserve the raw diagnostics, including known registry findings; never hide them.
process.stdout.write(output);
const reviewed = checkGlibAdvisories(events, record);
checkMacroAdvisories(events, macros);
console.log(`GLib registry reference: ${reviewed.join(', ')}; disposition requires verified vendored sources and optimized regression. Other packages remain covered by the full workspace advisory scan.`);
