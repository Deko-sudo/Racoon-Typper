#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(scriptDirectory, "..");

function read(path) {
  return readFileSync(join(repositoryRoot, path), "utf8");
}

function requiredMatch(value, pattern, description) {
  const match = value.match(pattern);
  if (!match) {
    throw new Error(`Unable to read ${description}`);
  }
  return match[1];
}

const cargoManifest = read("Cargo.toml");
const cargoVersion = requiredMatch(
  cargoManifest,
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
  "the workspace package version from Cargo.toml",
);
const tauriVersion = JSON.parse(read("crates/app/tauri.conf.json")).version;
const frontendPackage = JSON.parse(read("frontend/package.json"));
const frontendLock = JSON.parse(read("frontend/package-lock.json"));
const frontendLockVersion = frontendLock.packages?.[""]?.version;
const pkgbuildVersion = requiredMatch(
  read("PKGBUILD"),
  /^pkgver=([^\n]+)$/m,
  "pkgver from PKGBUILD",
);

const versions = {
  "Cargo.toml [workspace.package]": cargoVersion,
  "crates/app/tauri.conf.json": tauriVersion,
  "frontend/package.json": frontendPackage.version,
  "frontend/package-lock.json": frontendLockVersion,
  PKGBUILD: pkgbuildVersion,
};

const mismatches = Object.entries(versions).filter(
  ([, version]) => version !== cargoVersion,
);

if (mismatches.length > 0) {
  console.error("Project version mismatch detected:");
  for (const [source, version] of Object.entries(versions)) {
    console.error(`  ${source}: ${version ?? "<missing>"}`);
  }
  process.exit(1);
}

if (process.argv.includes("--print")) {
  console.log(cargoVersion);
} else {
  console.log(`Project version is consistent: ${cargoVersion}`);
}
