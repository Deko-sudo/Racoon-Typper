// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, symlinkSync, readFileSync, cpSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { verifyVendor, rustInventory, checkGlibAdvisories, checkMacroAdvisories } from './vendor-provenance.mjs';
const sha = value => createHash('sha256').update(value).digest('hex');

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'racoon-vendor-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, 'vendor/glib'), { recursive: true });
  writeFileSync(join(root, 'vendor/glib/LICENSE'), 'MIT'); writeFileSync(join(root, 'fix.patch'), 'fix');
  const record = { name: 'glib', version: '0.18.5', license: 'MIT', path: 'vendor/glib', files: { LICENSE: sha('MIT') }, patch: 'fix.patch', patch_sha256: sha('fix'), upstream_source: 'registry', upstream_archive_sha256: 'archive-hash' };
  const pkg = { name: 'glib', version: '0.18.5', license: 'MIT', id: 'path-glib', source: null, manifest_path: join(root, 'vendor/glib/Cargo.toml') };
  return { root, record, pkg };
}

test('local third-party crate remains in inventory with its license and provenance', t => {
  const { root, record, pkg } = fixture(t);
  const metadata = { workspace_members: ['app'], packages: [{ ...pkg, id: 'app', name: 'racoon-app' }, pkg] };
  const inventory = rustInventory(metadata, root, [record]);
  assert.equal(inventory.length, 1); assert.equal(inventory[0].license, 'MIT');
  assert.equal(inventory[0].source, 'vendored:vendor/glib');
  assert.equal(inventory[0].vendor.patch_sha256, sha('fix'));
  assert.throws(() => rustInventory(metadata, root, []), /Untracked local third-party/);
  assert.throws(() => rustInventory(metadata, root, [{ ...record, license: 'Apache-2.0' }]), /Untracked local third-party/);
});

test('source edits, extra/missing files and patch edits fail closed', t => {
  const { root, record } = fixture(t);
  verifyVendor(root, record);
  writeFileSync(join(root, 'vendor/glib/LICENSE'), 'changed');
  assert.throws(() => verifyVendor(root, record), /checksum mismatch/);
  writeFileSync(join(root, 'vendor/glib/LICENSE'), 'MIT');
  writeFileSync(join(root, 'vendor/glib/extra'), 'extra');
  assert.throws(() => verifyVendor(root, record), /file list changed/);
  rmSync(join(root, 'vendor/glib/extra')); writeFileSync(join(root, 'fix.patch'), 'changed');
  assert.throws(() => verifyVendor(root, record), /patch checksum mismatch/);
  rmSync(join(root, 'vendor/glib/LICENSE'));
  assert.throws(() => verifyVendor(root, record), /file list changed/);
});

test('untracked paths and symlinked vendor sources are rejected', t => {
  const { root, record } = fixture(t);
  assert.throws(() => verifyVendor(root, { ...record, path: '../other' }), /escapes/);
  symlinkSync(join(root, 'fix.patch'), join(root, 'vendor/glib/link'));
  assert.throws(() => verifyVendor(root, record), /symlinks/);
});

test('the checked-in GLib source matches the recorded two-line upstream backport', () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const record = JSON.parse(readFileSync(resolve(root, 'licenses/vendor-glib.json')));
  verifyVendor(root, record);
  const source = readFileSync(resolve(root, record.path, 'src/variant_iter.rs'), 'utf8');
  assert.match(source, /let mut p: \*mut libc::c_char = std::ptr::null_mut\(\);/);
  assert.match(source, /&mut p,/);
  const original = source.replace('let mut p: *mut libc::c_char', 'let p: *mut libc::c_char').replace('&mut p,', '&p,');
  assert.equal(sha(original), record.original_variant_sha256);
});

test('registry-reference gate retains the known finding and rejects new or missing GLib advisories', () => {
  const known = { fields: { advisory: { package: 'glib', id: 'RUSTSEC-2024-0429' } } };
  const disposition = { advisory_disposition: { 'RUSTSEC-2024-0429': 'verified backport' } };
  assert.deepEqual(checkGlibAdvisories([known], disposition), ['RUSTSEC-2024-0429']);
  assert.throws(() => checkGlibAdvisories([], disposition), /missing/);
  assert.throws(() => checkGlibAdvisories([known], {}), /Unreviewed/);
  assert.throws(() => checkGlibAdvisories([known, { advisory: { package: 'glib', id: 'RUSTSEC-FUTURE' } }], disposition), /RUSTSEC-FUTURE/);
});

test('desktop bundles retain the vendored license, copyright and provenance', () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const config = JSON.parse(readFileSync(resolve(root, 'crates/app/tauri.conf.json')));
  for (const resource of ['../../vendor/glib-0.18.5/LICENSE', '../../vendor/glib-0.18.5/COPYRIGHT', '../../licenses/vendor-glib.json']) {
    assert.ok(config.bundle.resources.includes(resource), `Missing GLib notice: ${resource}`);
  }
});

test('macro source provenance and bundled notices cover both local third-party crates', () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const config = JSON.parse(readFileSync(resolve(root, 'crates/app/tauri.conf.json')));
  for (const name of ['glib-macros', 'gtk3-macros']) {
    const record = JSON.parse(readFileSync(resolve(root, `licenses/vendor-${name}.json`)));
    verifyVendor(root, record);
    for (const resource of [`../../${record.path}/LICENSE`, `../../${record.path}/COPYRIGHT`, `../../licenses/vendor-${name}.json`]) {
      assert.ok(config.bundle.resources.includes(resource), `Missing macro notice: ${resource}`);
    }
  }
});

test('a registry finding for either vendored macro crate fails closed', () => {
  const records = [{ name: 'glib-macros' }, { name: 'gtk3-macros' }];
  checkMacroAdvisories([], records);
  for (const { name } of records) {
    assert.throws(() => checkMacroAdvisories([{ fields: { advisory: { package: name, id: 'RUSTSEC-FUTURE' } } }], records), /Unreviewed macro advisory/);
  }
});

test('each macro patch reverses to the recorded original archive files', t => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  for (const name of ['glib-macros', 'gtk3-macros']) {
    const record = JSON.parse(readFileSync(resolve(root, `licenses/vendor-${name}.json`)));
    verifyVendor(root, record);
    assert.deepEqual(Object.keys(record.original_files).sort(), Object.keys(record.files).sort());
    const directory = mkdtempSync(join(tmpdir(), 'racoon-macro-provenance-'));
    t.after(() => rmSync(directory, { recursive: true, force: true }));
    cpSync(resolve(root, record.path), directory, { recursive: true });
    execFileSync('git', ['apply', '--reverse', resolve(root, record.patch)], { cwd: directory });
    for (const [file, checksum] of Object.entries(record.original_files)) {
      assert.equal(sha(readFileSync(join(directory, file))), checksum, `${name}/${file}`);
    }
  }
});
