// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, lstatSync } from 'node:fs';
import { resolve, relative, dirname, isAbsolute, sep } from 'node:path';

const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
function inside(root, path) {
  const full = resolve(root, path);
  const rel = relative(root, full);
  if (!rel || rel.startsWith('..') || isAbsolute(rel)) throw new Error('Vendor path escapes its root');
  let cursor = root;
  for (const part of rel.split(sep)) {
    cursor = resolve(cursor, part);
    if (lstatSync(cursor).isSymbolicLink()) throw new Error('Vendor symlinks are not allowed');
  }
  return full;
}

export function verifyVendor(root, record) {
  const directory = inside(root, record.path);
  const files = [];
  function walk(path) {
    if (lstatSync(path).isSymbolicLink()) throw new Error('Vendor symlinks are not allowed');
    for (const entry of readdirSync(path, { withFileTypes: true })) {
      const file = resolve(path, entry.name);
      if (entry.isSymbolicLink()) throw new Error('Vendor symlinks are not allowed');
      if (entry.isDirectory()) walk(file);
      else if (entry.isFile()) files.push(relative(directory, file).split('\\').join('/'));
      else throw new Error('Unexpected vendor file type');
    }
  }
  walk(directory);
  if (JSON.stringify(files.sort()) !== JSON.stringify(Object.keys(record.files).sort())) throw new Error('Vendor file list changed');
  for (const [file, checksum] of Object.entries(record.files)) {
    if (hash(inside(directory, file)) !== checksum) throw new Error(`Vendor checksum mismatch: ${record.name}/${file}`);
  }
  if (hash(inside(root, record.patch)) !== record.patch_sha256) throw new Error('Vendor patch checksum mismatch');
}

export function rustInventory(metadata, root, records) {
  const members = new Set(metadata.workspace_members);
  const entries = [];
  for (const pkg of metadata.packages) {
    let vendor;
    if (!pkg.source) {
      if (members.has(pkg.id)) continue;
      vendor = records.find(record => resolve(root, record.path) === dirname(pkg.manifest_path));
      if (!vendor || vendor.name !== pkg.name || vendor.version !== pkg.version || vendor.license !== pkg.license) {
        throw new Error(`Untracked local third-party package: ${pkg.name}@${pkg.version}`);
      }
      verifyVendor(root, vendor);
    }
    entries.push({
      name: pkg.name, version: pkg.version, license: pkg.license || 'UNKNOWN',
      source: pkg.source || `vendored:${vendor.path}`,
      repository: pkg.repository || null, authors: pkg.authors || [],
      ...(vendor ? { vendor: { path: vendor.path, upstream_source: vendor.upstream_source,
        upstream_archive_sha256: vendor.upstream_archive_sha256, patch: vendor.patch, patch_sha256: vendor.patch_sha256 } } : {}),
    });
  }
  return entries.sort((a, b) => `${a.name}@${a.version}`.localeCompare(`${b.name}@${b.version}`));
}

// A path dependency is not covered by cargo-deny's crates.io advisory matcher.
// Keep a registry reference scan so future GLib advisories remain release inputs.
export function checkGlibAdvisories(events, record) {
  const glib = events.map(event => event.fields ?? event)
    .filter(event => event.advisory?.package === 'glib');
  if (!glib.some(event => event.advisory.id === 'RUSTSEC-2024-0429')) {
    throw new Error('Expected GLib registry advisory missing; review the reference scan');
  }
  for (const event of glib) {
    if (event.advisory.id !== 'RUSTSEC-2024-0429' || !record.advisory_disposition?.[event.advisory.id]) {
      throw new Error(`Unreviewed GLib advisory: ${event.advisory.id}`);
    }
  }
  return glib.map(event => event.advisory.id);
}

// These diagnostic-only backports do not resolve any advisory against the
// parent macro crates themselves. Any registry finding needs a fresh review.
export function checkMacroAdvisories(events, records) {
  const names = new Set(records.map(record => record.name));
  for (const event of events.map(event => event.fields ?? event)) {
    if (names.has(event.advisory?.package)) {
      throw new Error(`Unreviewed macro advisory: ${event.advisory.package}/${event.advisory.id}`);
    }
  }
}
